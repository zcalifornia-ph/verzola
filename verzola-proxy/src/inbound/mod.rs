use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub const DEFAULT_MAX_LINE_LEN: usize = 4096;

#[derive(Debug, Clone)]
pub struct ListenerConfig {
    pub bind_addr: SocketAddr,
    pub banner_host: String,
    pub advertise_starttls: bool,
    pub max_line_len: usize,
    pub postfix_upstream_addr: Option<SocketAddr>,
}

impl ListenerConfig {
    pub fn validate(&self) -> io::Result<()> {
        if self.banner_host.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "banner_host must not be empty",
            ));
        }

        if self.max_line_len < 512 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "max_line_len must be at least 512 bytes",
            ));
        }

        if self.postfix_upstream_addr == Some(self.bind_addr) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "postfix_upstream_addr must not equal bind_addr",
            ));
        }

        Ok(())
    }
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:2525"
                .parse()
                .expect("default inbound socket address must parse"),
            banner_host: "localhost".to_string(),
            advertise_starttls: true,
            max_line_len: DEFAULT_MAX_LINE_LEN,
            postfix_upstream_addr: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TlsUpgradeError {
    Temporary(String),
}

impl Display for TlsUpgradeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsUpgradeError::Temporary(message) => f.write_str(message),
        }
    }
}

pub trait TlsUpgrader: Send + Sync + 'static {
    fn upgrade(&self, stream: &mut TcpStream) -> Result<(), TlsUpgradeError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoopTlsUpgrader;

impl TlsUpgrader for NoopTlsUpgrader {
    fn upgrade(&self, _stream: &mut TcpStream) -> Result<(), TlsUpgradeError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SessionSummary {
    pub command_count: usize,
    pub protocol_errors: usize,
    pub tls_negotiated: bool,
}

pub struct InboundListener<U>
where
    U: TlsUpgrader,
{
    listener: TcpListener,
    config: ListenerConfig,
    tls_upgrader: Arc<U>,
}

impl<U> InboundListener<U>
where
    U: TlsUpgrader,
{
    pub fn bind(config: ListenerConfig, tls_upgrader: U) -> io::Result<Self> {
        config.validate()?;
        let listener = TcpListener::bind(config.bind_addr)?;
        Ok(Self {
            listener,
            config,
            tls_upgrader: Arc::new(tls_upgrader),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn serve_one(&self) -> io::Result<SessionSummary> {
        let (mut stream, _) = self.listener.accept()?;
        handle_session(&mut stream, &self.config, self.tls_upgrader.as_ref())
    }

    pub fn serve_n(&self, session_count: usize) -> io::Result<Vec<SessionSummary>> {
        let mut handles = Vec::with_capacity(session_count);

        for _ in 0..session_count {
            let (mut stream, _) = self.listener.accept()?;
            let config = self.config.clone();
            let tls_upgrader = Arc::clone(&self.tls_upgrader);
            handles.push(thread::spawn(move || {
                handle_session(&mut stream, &config, tls_upgrader.as_ref())
            }));
        }

        let mut summaries = Vec::with_capacity(session_count);
        for handle in handles {
            let summary = handle
                .join()
                .map_err(|_| io::Error::new(ErrorKind::Other, "session worker thread panicked"))??;
            summaries.push(summary);
        }

        Ok(summaries)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct SessionState {
    tls_active: bool,
    ehlo_seen: bool,
    command_count: usize,
    protocol_errors: usize,
}

#[derive(Debug)]
struct SmtpReply {
    code: u16,
    lines: Vec<String>,
}

struct PostfixRelay {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl PostfixRelay {
    fn connect(upstream_addr: SocketAddr, ehlo_host: &str) -> io::Result<Self> {
        let mut writer = TcpStream::connect(upstream_addr)?;
        let mut reader = BufReader::new(writer.try_clone()?);

        let banner_reply = read_smtp_reply(&mut reader)?;
        if banner_reply.code / 100 != 2 {
            return Err(io::Error::new(
                ErrorKind::ConnectionAborted,
                format!(
                    "upstream Postfix banner was non-2xx ({}): {}",
                    banner_reply.code,
                    banner_reply.lines.join(" | ")
                ),
            ));
        }

        write_command_line(&mut writer, &format!("EHLO {}", ehlo_host))?;
        let ehlo_reply = read_smtp_reply(&mut reader)?;
        if ehlo_reply.code / 100 != 2 {
            return Err(io::Error::new(
                ErrorKind::ConnectionAborted,
                format!(
                    "upstream Postfix EHLO was non-2xx ({}): {}",
                    ehlo_reply.code,
                    ehlo_reply.lines.join(" | ")
                ),
            ));
        }

        Ok(Self { writer, reader })
    }

    fn relay_command(&mut self, command_line: &str) -> io::Result<SmtpReply> {
        write_command_line(&mut self.writer, command_line)?;
        read_smtp_reply(&mut self.reader)
    }

    fn relay_data_block(
        &mut self,
        client_reader: &mut BufReader<TcpStream>,
        max_line_len: usize,
    ) -> io::Result<SmtpReply> {
        loop {
            let mut line = String::new();
            let bytes_read = client_reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "connection closed during DATA relay",
                ));
            }

            if line.len() > max_line_len {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "DATA line exceeds max_line_len during relay",
                ));
            }

            self.writer.write_all(line.as_bytes())?;
            self.writer.flush()?;

            if is_data_terminator(&line) {
                break;
            }
        }

        read_smtp_reply(&mut self.reader)
    }
}

fn handle_session<U>(
    stream: &mut TcpStream,
    config: &ListenerConfig,
    tls_upgrader: &U,
) -> io::Result<SessionSummary>
where
    U: TlsUpgrader,
{
    write_reply(stream, 220, &format!("{} ESMTP VERZOLA", config.banner_host))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut state = SessionState::default();
    let mut relay: Option<PostfixRelay> = None;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        if line.len() > config.max_line_len {
            state.protocol_errors += 1;
            write_reply(stream, 500, "5.5.2 Line too long")?;
            continue;
        }

        let command_line = line.trim_end_matches(['\r', '\n']);
        if command_line.is_empty() {
            state.protocol_errors += 1;
            write_reply(stream, 500, "5.5.2 Empty command")?;
            continue;
        }

        state.command_count += 1;
        let (verb, argument) = split_command(command_line);

        match verb.as_str() {
            "EHLO" | "HELO" => {
                state.ehlo_seen = true;
                let greeting_target = if argument.is_empty() { "client" } else { argument };
                let mut lines = vec![format!("{} greets {}", config.banner_host, greeting_target)];
                if config.advertise_starttls && !state.tls_active {
                    lines.push("STARTTLS".to_string());
                }
                lines.push("SIZE 10485760".to_string());
                write_multiline_reply(stream, 250, &lines)?;
            }
            "STARTTLS" => {
                if !config.advertise_starttls {
                    state.protocol_errors += 1;
                    write_reply(stream, 502, "5.5.1 STARTTLS not supported")?;
                    continue;
                }

                if state.tls_active {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, "5.5.1 TLS already active")?;
                    continue;
                }

                if !state.ehlo_seen {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, "5.5.1 Send EHLO before STARTTLS")?;
                    continue;
                }

                write_reply(stream, 220, "Ready to start TLS")?;
                match tls_upgrader.upgrade(stream) {
                    Ok(()) => {
                        state.tls_active = true;
                        state.ehlo_seen = false;
                        relay = None;
                    }
                    Err(error) => {
                        state.protocol_errors += 1;
                        write_reply(
                            stream,
                            454,
                            &format!("4.7.0 TLS not available due to temporary reason: {}", error),
                        )?;
                    }
                }
            }
            "MAIL" => {
                if !can_process_mail_command(&state) {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, required_ehlo_message(&state))?;
                    continue;
                }

                if config.postfix_upstream_addr.is_some() {
                    let mail_reply =
                        match relay_command_to_postfix(&mut relay, config, command_line) {
                            Ok(reply) => reply,
                            Err(error) => {
                                relay = None;
                                state.protocol_errors += 1;
                                write_reply(
                                    stream,
                                    451,
                                    &format!("4.4.0 Postfix relay unavailable: {}", error),
                                )?;
                                continue;
                            }
                        };
                    write_smtp_reply(stream, &mail_reply)?;
                } else {
                    write_reply(stream, 250, "2.1.0 Sender OK")?;
                }
            }
            "RCPT" => {
                if !can_process_mail_command(&state) {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, required_ehlo_message(&state))?;
                    continue;
                }

                if config.postfix_upstream_addr.is_some() {
                    let rcpt_reply =
                        match relay_command_to_postfix(&mut relay, config, command_line) {
                            Ok(reply) => reply,
                            Err(error) => {
                                relay = None;
                                state.protocol_errors += 1;
                                write_reply(
                                    stream,
                                    451,
                                    &format!("4.4.0 Postfix relay unavailable: {}", error),
                                )?;
                                continue;
                            }
                        };
                    write_smtp_reply(stream, &rcpt_reply)?;
                } else {
                    write_reply(stream, 250, "2.1.5 Recipient OK")?;
                }
            }
            "DATA" => {
                if !can_process_mail_command(&state) {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, required_ehlo_message(&state))?;
                    continue;
                }

                if config.postfix_upstream_addr.is_some() {
                    let data_reply = match relay_command_to_postfix(&mut relay, config, command_line)
                    {
                        Ok(reply) => reply,
                        Err(error) => {
                            relay = None;
                            state.protocol_errors += 1;
                            write_reply(
                                stream,
                                451,
                                &format!("4.4.0 Postfix relay unavailable: {}", error),
                            )?;
                            continue;
                        }
                    };
                    write_smtp_reply(stream, &data_reply)?;

                    if data_reply.code / 100 != 3 {
                        continue;
                    }

                    let final_data_reply = match relay.as_mut() {
                        Some(postfix_relay) => {
                            postfix_relay.relay_data_block(&mut reader, config.max_line_len)
                        }
                        None => Err(io::Error::new(
                            ErrorKind::NotConnected,
                            "relay state missing after DATA command",
                        )),
                    };

                    match final_data_reply {
                        Ok(reply) => write_smtp_reply(stream, &reply)?,
                        Err(error) => {
                            relay = None;
                            state.protocol_errors += 1;
                            write_reply(
                                stream,
                                451,
                                &format!("4.3.0 DATA relay failure: {}", error),
                            )?;
                        }
                    }
                } else {
                    write_reply(stream, 354, "End data with <CR><LF>.<CR><LF>")?;
                    if let Err(error) = consume_data_block(&mut reader, config.max_line_len) {
                        state.protocol_errors += 1;
                        write_reply(stream, 451, &format!("4.3.0 DATA read failure: {}", error))?;
                        continue;
                    }
                    write_reply(stream, 250, "2.0.0 Queued")?;
                }
            }
            "RSET" => {
                if relay.is_some() {
                    match relay_command_to_postfix(&mut relay, config, command_line) {
                        Ok(reply) => write_smtp_reply(stream, &reply)?,
                        Err(error) => {
                            relay = None;
                            state.protocol_errors += 1;
                            write_reply(
                                stream,
                                451,
                                &format!("4.4.0 Postfix relay unavailable: {}", error),
                            )?;
                        }
                    }
                } else {
                    write_reply(stream, 250, "2.0.0 Reset state")?;
                }
            }
            "NOOP" => {
                if relay.is_some() {
                    match relay_command_to_postfix(&mut relay, config, command_line) {
                        Ok(reply) => write_smtp_reply(stream, &reply)?,
                        Err(error) => {
                            relay = None;
                            state.protocol_errors += 1;
                            write_reply(
                                stream,
                                451,
                                &format!("4.4.0 Postfix relay unavailable: {}", error),
                            )?;
                        }
                    }
                } else {
                    write_reply(stream, 250, "2.0.0 OK")?;
                }
            }
            "QUIT" => {
                if relay.is_some() {
                    match relay_command_to_postfix(&mut relay, config, command_line) {
                        Ok(reply) => write_smtp_reply(stream, &reply)?,
                        Err(_) => write_reply(stream, 221, "2.0.0 Bye")?,
                    }
                } else {
                    write_reply(stream, 221, "2.0.0 Bye")?;
                }
                break;
            }
            _ => {
                state.protocol_errors += 1;
                write_reply(stream, 502, "5.5.1 Command not implemented")?;
            }
        }
    }

    Ok(SessionSummary {
        command_count: state.command_count,
        protocol_errors: state.protocol_errors,
        tls_negotiated: state.tls_active,
    })
}

fn can_process_mail_command(state: &SessionState) -> bool {
    state.ehlo_seen
}

fn required_ehlo_message(state: &SessionState) -> &'static str {
    if state.tls_active {
        "5.5.1 Send EHLO after STARTTLS"
    } else {
        "5.5.1 Send EHLO before MAIL"
    }
}

fn consume_data_block(reader: &mut BufReader<TcpStream>, max_line_len: usize) -> io::Result<()> {
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "connection closed during DATA",
            ));
        }
        if line.len() > max_line_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "DATA line too long",
            ));
        }
        if line == ".\r\n" || line == ".\n" {
            return Ok(());
        }
    }
}

fn split_command(line: &str) -> (String, &str) {
    let mut parts = line.splitn(2, |character: char| character.is_whitespace());
    let verb = parts.next().unwrap_or("").trim().to_ascii_uppercase();
    let argument = parts.next().unwrap_or("").trim();
    (verb, argument)
}

fn relay_command_to_postfix(
    relay: &mut Option<PostfixRelay>,
    config: &ListenerConfig,
    command_line: &str,
) -> io::Result<SmtpReply> {
    let postfix_relay = ensure_postfix_relay(relay, config)?;
    postfix_relay.relay_command(command_line)
}

fn ensure_postfix_relay<'a>(
    relay: &'a mut Option<PostfixRelay>,
    config: &ListenerConfig,
) -> io::Result<&'a mut PostfixRelay> {
    if relay.is_none() {
        let upstream_addr = config.postfix_upstream_addr.ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidInput,
                "postfix_upstream_addr is required for relay mode",
            )
        })?;
        *relay = Some(PostfixRelay::connect(upstream_addr, &config.banner_host)?);
    }

    match relay {
        Some(postfix_relay) => Ok(postfix_relay),
        None => Err(io::Error::new(
            ErrorKind::NotConnected,
            "relay initialization failed",
        )),
    }
}

fn read_smtp_reply(reader: &mut BufReader<TcpStream>) -> io::Result<SmtpReply> {
    let mut lines = Vec::new();
    let mut code: Option<u16> = None;

    loop {
        let mut raw = String::new();
        let bytes_read = reader.read_line(&mut raw)?;
        if bytes_read == 0 {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "upstream closed connection while reading SMTP reply",
            ));
        }

        let line = raw.trim_end_matches(['\r', '\n']).to_string();
        let line_bytes = line.as_bytes();
        if line_bytes.len() < 3 || !line_bytes[..3].iter().all(|byte| byte.is_ascii_digit()) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid SMTP reply line from upstream: {}", line),
            ));
        }

        let parsed_code = line[..3].parse::<u16>().map_err(|error| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid SMTP reply code from upstream: {}", error),
            )
        })?;

        if let Some(expected_code) = code {
            if expected_code != parsed_code {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "inconsistent SMTP reply code from upstream: expected {}, got {}",
                        expected_code, parsed_code
                    ),
                ));
            }
        } else {
            code = Some(parsed_code);
        }

        let separator = if line_bytes.len() > 3 {
            line_bytes[3]
        } else {
            b' '
        };
        lines.push(line);
        if separator == b' ' {
            break;
        }
        if separator != b'-' {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "invalid SMTP multiline reply separator from upstream",
            ));
        }
    }

    Ok(SmtpReply {
        code: code.unwrap_or(500),
        lines,
    })
}

fn write_command_line(stream: &mut TcpStream, line: &str) -> io::Result<()> {
    write!(stream, "{}\r\n", line)?;
    stream.flush()
}

fn write_smtp_reply(stream: &mut TcpStream, reply: &SmtpReply) -> io::Result<()> {
    for line in &reply.lines {
        write!(stream, "{}\r\n", line)?;
    }

    stream.flush()
}

fn is_data_terminator(line: &str) -> bool {
    line == ".\r\n" || line == ".\n"
}

fn write_reply(stream: &mut TcpStream, code: u16, message: &str) -> io::Result<()> {
    write!(stream, "{} {}\r\n", code, message)?;
    stream.flush()
}

fn write_multiline_reply(stream: &mut TcpStream, code: u16, lines: &[String]) -> io::Result<()> {
    if lines.is_empty() {
        return write_reply(stream, code, "OK");
    }

    for (index, line) in lines.iter().enumerate() {
        let separator = if index + 1 == lines.len() { ' ' } else { '-' };
        write!(stream, "{}{}{}\r\n", code, separator, line)?;
    }

    stream.flush()
}
