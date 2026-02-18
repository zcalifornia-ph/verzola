use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub const DEFAULT_MAX_LINE_LEN: usize = 4096;

#[derive(Debug, Clone)]
pub struct OutboundListenerConfig {
    pub bind_addr: SocketAddr,
    pub banner_host: String,
    pub max_line_len: usize,
}

impl OutboundListenerConfig {
    pub fn validate(&self) -> io::Result<()> {
        if self.banner_host.trim().is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "banner_host must not be empty",
            ));
        }

        if self.max_line_len < 512 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "max_line_len must be at least 512 bytes",
            ));
        }

        Ok(())
    }
}

impl Default for OutboundListenerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:10025"
                .parse()
                .expect("default outbound socket address must parse"),
            banner_host: "localhost".to_string(),
            max_line_len: DEFAULT_MAX_LINE_LEN,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MxCandidate {
    pub preference: u16,
    pub exchange: String,
    pub address: SocketAddr,
}

impl MxCandidate {
    pub fn new(
        preference: u16,
        exchange: impl Into<String>,
        address: SocketAddr,
    ) -> io::Result<Self> {
        let exchange = exchange.into();
        if exchange.trim().is_empty() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "mx exchange must not be empty",
            ));
        }

        Ok(Self {
            preference,
            exchange,
            address,
        })
    }
}

#[derive(Debug, Clone)]
pub enum MxResolutionError {
    Temporary(String),
}

impl Display for MxResolutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MxResolutionError::Temporary(message) => f.write_str(message),
        }
    }
}

pub trait MxResolver: Send + Sync + 'static {
    fn resolve(&self, recipient_domain: &str) -> Result<Vec<MxCandidate>, MxResolutionError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMxResolver;

impl MxResolver for NoopMxResolver {
    fn resolve(&self, _recipient_domain: &str) -> Result<Vec<MxCandidate>, MxResolutionError> {
        Err(MxResolutionError::Temporary(
            "no MX resolver configured".to_string(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutboundSessionSummary {
    pub command_count: usize,
    pub protocol_errors: usize,
    pub temporary_failures: usize,
    pub resolver_lookups: usize,
    pub mx_candidates_attempted: usize,
    pub remote_session_established: bool,
    pub selected_mx: Option<String>,
    pub selected_recipient_domain: Option<String>,
}

pub struct OutboundListener<R>
where
    R: MxResolver,
{
    listener: TcpListener,
    config: OutboundListenerConfig,
    resolver: Arc<R>,
}

impl<R> OutboundListener<R>
where
    R: MxResolver,
{
    pub fn bind(config: OutboundListenerConfig, resolver: R) -> io::Result<Self> {
        config.validate()?;
        let listener = TcpListener::bind(config.bind_addr)?;
        Ok(Self {
            listener,
            config,
            resolver: Arc::new(resolver),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn serve_one(&self) -> io::Result<OutboundSessionSummary> {
        let (mut stream, _) = self.listener.accept()?;
        handle_session(&mut stream, &self.config, self.resolver.as_ref())
    }

    pub fn serve_n(&self, session_count: usize) -> io::Result<Vec<OutboundSessionSummary>> {
        let mut handles = Vec::with_capacity(session_count);

        for _ in 0..session_count {
            let (mut stream, _) = self.listener.accept()?;
            let config = self.config.clone();
            let resolver = Arc::clone(&self.resolver);
            handles.push(thread::spawn(move || {
                handle_session(&mut stream, &config, resolver.as_ref())
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

#[derive(Debug, Default)]
struct SessionState {
    ehlo_seen: bool,
    command_count: usize,
    protocol_errors: usize,
    temporary_failures: usize,
    resolver_lookups: usize,
    mx_candidates_attempted: usize,
    remote_session_established: bool,
    selected_mx: Option<String>,
    selected_recipient_domain: Option<String>,
    staged_mail_from: Option<String>,
    recipient_domain: Option<String>,
    recipient_count: usize,
}

#[derive(Debug)]
struct SmtpReply {
    code: u16,
    lines: Vec<String>,
}

struct RemoteMxRelay {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
    exchange: String,
}

impl RemoteMxRelay {
    fn connect(candidate: &MxCandidate, ehlo_host: &str, mail_command: &str) -> io::Result<Self> {
        let mut writer = TcpStream::connect(candidate.address)?;
        let mut reader = BufReader::new(writer.try_clone()?);

        let banner_reply = read_smtp_reply(&mut reader)?;
        if banner_reply.code / 100 != 2 {
            return Err(io::Error::new(
                ErrorKind::ConnectionAborted,
                format!(
                    "remote MX banner was non-2xx ({}): {}",
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
                    "remote MX EHLO was non-2xx ({}): {}",
                    ehlo_reply.code,
                    ehlo_reply.lines.join(" | ")
                ),
            ));
        }

        write_command_line(&mut writer, mail_command)?;
        let mail_reply = read_smtp_reply(&mut reader)?;
        if mail_reply.code / 100 != 2 {
            return Err(io::Error::new(
                ErrorKind::ConnectionAborted,
                format!(
                    "remote MX MAIL was non-2xx ({}): {}",
                    mail_reply.code,
                    mail_reply.lines.join(" | ")
                ),
            ));
        }

        Ok(Self {
            writer,
            reader,
            exchange: candidate.exchange.clone(),
        })
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

fn handle_session<R>(
    stream: &mut TcpStream,
    config: &OutboundListenerConfig,
    resolver: &R,
) -> io::Result<OutboundSessionSummary>
where
    R: MxResolver,
{
    write_reply(stream, 220, &format!("{} ESMTP VERZOLA", config.banner_host))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut state = SessionState::default();
    let mut relay: Option<RemoteMxRelay> = None;

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
                let greeting_target = if argument.is_empty() {
                    "postfix"
                } else {
                    argument
                };
                let lines = vec![
                    format!("{} greets {}", config.banner_host, greeting_target),
                    "SIZE 10485760".to_string(),
                ];
                write_multiline_reply(stream, 250, &lines)?;
            }
            "MAIL" => {
                if !state.ehlo_seen {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, "5.5.1 Send EHLO before MAIL")?;
                    continue;
                }

                if !is_mail_from_argument(argument) {
                    state.protocol_errors += 1;
                    write_reply(stream, 501, "5.5.4 MAIL requires FROM:<address>")?;
                    continue;
                }

                state.staged_mail_from = Some(command_line.to_string());
                state.recipient_domain = None;
                state.recipient_count = 0;
                state.selected_mx = None;
                relay = None;

                write_reply(stream, 250, "2.1.0 Sender staged for outbound relay")?;
            }
            "RCPT" => {
                if !state.ehlo_seen {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, "5.5.1 Send EHLO before RCPT")?;
                    continue;
                }

                let staged_mail_command = match state.staged_mail_from.clone() {
                    Some(mail_command) => mail_command,
                    None => {
                        state.protocol_errors += 1;
                        write_reply(stream, 503, "5.5.1 Send MAIL before RCPT")?;
                        continue;
                    }
                };

                let domain = match parse_recipient_domain(argument) {
                    Some(domain) => domain,
                    None => {
                        state.protocol_errors += 1;
                        write_reply(stream, 501, "5.1.3 Bad recipient address syntax")?;
                        continue;
                    }
                };

                if let Some(active_domain) = state.recipient_domain.as_deref() {
                    if active_domain != domain {
                        state.temporary_failures += 1;
                        write_reply(
                            stream,
                            451,
                            "4.5.3 Mixed recipient domains are not supported in this bolt",
                        )?;
                        continue;
                    }
                } else {
                    state.recipient_domain = Some(domain.clone());
                }

                let outbound_relay = match ensure_remote_relay(
                    &mut relay,
                    &mut state,
                    config,
                    resolver,
                    &domain,
                    &staged_mail_command,
                ) {
                    Ok(outbound_relay) => outbound_relay,
                    Err(error) => {
                        relay = None;
                        state.temporary_failures += 1;
                        write_reply(
                            stream,
                            451,
                            &format!("4.4.0 Outbound MX temporarily unavailable: {}", error),
                        )?;
                        continue;
                    }
                };

                let rcpt_reply = match outbound_relay.relay_command(command_line) {
                    Ok(reply) => reply,
                    Err(error) => {
                        relay = None;
                        state.temporary_failures += 1;
                        write_reply(
                            stream,
                            451,
                            &format!("4.4.0 Remote RCPT relay failure: {}", error),
                        )?;
                        continue;
                    }
                };

                if rcpt_reply.code / 100 == 2 {
                    state.recipient_count += 1;
                }

                write_smtp_reply(stream, &rcpt_reply)?;
            }
            "DATA" => {
                if !state.ehlo_seen {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, "5.5.1 Send EHLO before DATA")?;
                    continue;
                }

                if state.staged_mail_from.is_none() {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, "5.5.1 Send MAIL before DATA")?;
                    continue;
                }

                if state.recipient_count == 0 {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, "5.5.1 Send RCPT before DATA")?;
                    continue;
                }

                let outbound_relay = match relay.as_mut() {
                    Some(outbound_relay) => outbound_relay,
                    None => {
                        state.temporary_failures += 1;
                        write_reply(stream, 451, "4.4.0 Outbound relay session is unavailable")?;
                        continue;
                    }
                };

                let data_reply = match outbound_relay.relay_command(command_line) {
                    Ok(reply) => reply,
                    Err(error) => {
                        relay = None;
                        state.temporary_failures += 1;
                        write_reply(
                            stream,
                            451,
                            &format!("4.4.0 Remote DATA relay failure: {}", error),
                        )?;
                        continue;
                    }
                };

                write_smtp_reply(stream, &data_reply)?;

                if data_reply.code / 100 != 3 {
                    continue;
                }

                let final_data_reply =
                    match outbound_relay.relay_data_block(&mut reader, config.max_line_len) {
                        Ok(reply) => reply,
                        Err(error) => {
                            relay = None;
                            state.temporary_failures += 1;
                            write_reply(
                                stream,
                                451,
                                &format!("4.4.0 Remote DATA payload relay failure: {}", error),
                            )?;
                            continue;
                        }
                    };

                if final_data_reply.code / 100 == 2 {
                    state.staged_mail_from = None;
                    state.recipient_domain = None;
                    state.recipient_count = 0;
                }

                write_smtp_reply(stream, &final_data_reply)?;
            }
            "RSET" => {
                state.staged_mail_from = None;
                state.recipient_domain = None;
                state.recipient_count = 0;

                if relay.is_some() {
                    match relay.as_mut().unwrap().relay_command(command_line) {
                        Ok(reply) => write_smtp_reply(stream, &reply)?,
                        Err(error) => {
                            relay = None;
                            state.temporary_failures += 1;
                            write_reply(
                                stream,
                                451,
                                &format!("4.4.0 Remote RSET relay failure: {}", error),
                            )?;
                        }
                    }
                } else {
                    write_reply(stream, 250, "2.0.0 Reset state")?;
                }
            }
            "NOOP" => {
                if relay.is_some() {
                    match relay.as_mut().unwrap().relay_command(command_line) {
                        Ok(reply) => write_smtp_reply(stream, &reply)?,
                        Err(error) => {
                            relay = None;
                            state.temporary_failures += 1;
                            write_reply(
                                stream,
                                451,
                                &format!("4.4.0 Remote NOOP relay failure: {}", error),
                            )?;
                        }
                    }
                } else {
                    write_reply(stream, 250, "2.0.0 OK")?;
                }
            }
            "QUIT" => {
                if relay.is_some() {
                    match relay.as_mut().unwrap().relay_command(command_line) {
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

    Ok(OutboundSessionSummary {
        command_count: state.command_count,
        protocol_errors: state.protocol_errors,
        temporary_failures: state.temporary_failures,
        resolver_lookups: state.resolver_lookups,
        mx_candidates_attempted: state.mx_candidates_attempted,
        remote_session_established: state.remote_session_established,
        selected_mx: state.selected_mx,
        selected_recipient_domain: state.selected_recipient_domain,
    })
}

fn ensure_remote_relay<'a, R>(
    relay: &'a mut Option<RemoteMxRelay>,
    state: &mut SessionState,
    config: &OutboundListenerConfig,
    resolver: &R,
    recipient_domain: &str,
    mail_command: &str,
) -> io::Result<&'a mut RemoteMxRelay>
where
    R: MxResolver,
{
    if relay.is_none() {
        state.resolver_lookups += 1;

        let mut candidates = resolver
            .resolve(recipient_domain)
            .map_err(mx_resolution_error_to_io)?;

        if candidates.is_empty() {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("resolver returned no MX records for {}", recipient_domain),
            ));
        }

        candidates.sort_by(|left, right| compare_mx_candidates(left, right));

        let mut last_error: Option<io::Error> = None;
        for candidate in candidates {
            state.mx_candidates_attempted += 1;

            match RemoteMxRelay::connect(&candidate, &config.banner_host, mail_command) {
                Ok(outbound_relay) => {
                    state.remote_session_established = true;
                    state.selected_mx = Some(outbound_relay.exchange.clone());
                    state.selected_recipient_domain = Some(recipient_domain.to_string());
                    *relay = Some(outbound_relay);
                    break;
                }
                Err(error) => {
                    last_error = Some(io::Error::new(
                        ErrorKind::ConnectionAborted,
                        format!("candidate {} failed: {}", candidate.exchange, error),
                    ));
                }
            }
        }

        if relay.is_none() {
            return Err(last_error.unwrap_or_else(|| {
                io::Error::new(
                    ErrorKind::ConnectionAborted,
                    "all MX candidate connection attempts failed",
                )
            }));
        }
    }

    match relay {
        Some(outbound_relay) => Ok(outbound_relay),
        None => Err(io::Error::new(
            ErrorKind::NotConnected,
            "outbound relay initialization failed",
        )),
    }
}

fn parse_recipient_domain(argument: &str) -> Option<String> {
    let trimmed = argument.trim();
    if trimmed.is_empty() {
        return None;
    }

    let value = if starts_with_case_insensitive(trimmed, "TO:") {
        &trimmed[3..]
    } else {
        trimmed
    };

    let address_token = value.trim().split_whitespace().next()?;
    let address = address_token.trim_matches(['<', '>']);
    let (_, domain) = address.rsplit_once('@')?;

    let domain = domain.trim().trim_end_matches('>');
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_ascii_lowercase())
    }
}

fn is_mail_from_argument(argument: &str) -> bool {
    let trimmed = argument.trim();
    if !starts_with_case_insensitive(trimmed, "FROM:") {
        return false;
    }

    let value = trimmed[5..].trim();
    !value.is_empty()
}

fn starts_with_case_insensitive(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .map(|head| head.eq_ignore_ascii_case(prefix))
        .unwrap_or(false)
}

fn compare_mx_candidates(left: &MxCandidate, right: &MxCandidate) -> Ordering {
    left.preference
        .cmp(&right.preference)
        .then_with(|| left.exchange.cmp(&right.exchange))
}

fn mx_resolution_error_to_io(error: MxResolutionError) -> io::Error {
    match error {
        MxResolutionError::Temporary(message) => io::Error::new(ErrorKind::WouldBlock, message),
    }
}

fn split_command(line: &str) -> (String, &str) {
    let mut parts = line.splitn(2, |character: char| character.is_whitespace());
    let verb = parts.next().unwrap_or("").trim().to_ascii_uppercase();
    let argument = parts.next().unwrap_or("").trim();
    (verb, argument)
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
                "remote side closed connection while reading SMTP reply",
            ));
        }

        let line = raw.trim_end_matches(['\r', '\n']).to_string();
        let line_bytes = line.as_bytes();

        if line_bytes.len() < 3 || !line_bytes[..3].iter().all(|byte| byte.is_ascii_digit()) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid SMTP reply line: {}", line),
            ));
        }

        let parsed_code = line[..3].parse::<u16>().map_err(|error| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid SMTP reply code: {}", error),
            )
        })?;

        if let Some(expected_code) = code {
            if expected_code != parsed_code {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "inconsistent SMTP reply codes in multiline response: expected {}, got {}",
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
                "invalid SMTP multiline reply separator",
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
