use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

pub const DEFAULT_MAX_LINE_LEN: usize = 4096;

#[derive(Debug, Clone)]
pub struct ListenerConfig {
    pub bind_addr: SocketAddr,
    pub banner_host: String,
    pub advertise_starttls: bool,
    pub max_line_len: usize,
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
    tls_upgrader: U,
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
            tls_upgrader,
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn serve_one(&self) -> io::Result<SessionSummary> {
        let (mut stream, _) = self.listener.accept()?;
        handle_session(&mut stream, &self.config, &self.tls_upgrader)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct SessionState {
    tls_active: bool,
    ehlo_seen: bool,
    command_count: usize,
    protocol_errors: usize,
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

                write_reply(stream, 250, "2.1.0 Sender OK")?;
            }
            "RCPT" => {
                if !can_process_mail_command(&state) {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, required_ehlo_message(&state))?;
                    continue;
                }

                write_reply(stream, 250, "2.1.5 Recipient OK")?;
            }
            "DATA" => {
                if !can_process_mail_command(&state) {
                    state.protocol_errors += 1;
                    write_reply(stream, 503, required_ehlo_message(&state))?;
                    continue;
                }

                write_reply(stream, 354, "End data with <CR><LF>.<CR><LF>")?;
                if let Err(error) = consume_data_block(&mut reader, config.max_line_len) {
                    state.protocol_errors += 1;
                    write_reply(stream, 451, &format!("4.3.0 DATA read failure: {}", error))?;
                    continue;
                }
                write_reply(stream, 250, "2.0.0 Queued")?;
            }
            "RSET" => {
                write_reply(stream, 250, "2.0.0 Reset state")?;
            }
            "NOOP" => {
                write_reply(stream, 250, "2.0.0 OK")?;
            }
            "QUIT" => {
                write_reply(stream, 221, "2.0.0 Bye")?;
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
