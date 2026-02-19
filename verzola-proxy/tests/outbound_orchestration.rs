use std::collections::HashMap;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use verzola_proxy::outbound::{
    MxCandidate, MxResolutionError, MxResolver, OutboundListener, OutboundListenerConfig,
    OutboundSessionSummary,
};

#[derive(Debug, Clone)]
struct StaticResolver {
    candidates_by_domain: HashMap<String, Vec<MxCandidate>>,
}

impl MxResolver for StaticResolver {
    fn resolve(&self, recipient_domain: &str) -> Result<Vec<MxCandidate>, MxResolutionError> {
        self.candidates_by_domain
            .get(recipient_domain)
            .cloned()
            .ok_or_else(|| {
                MxResolutionError::Temporary(format!(
                    "no MX records found for {}",
                    recipient_domain
                ))
            })
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct RemoteMxSessionStats {
    message_count: usize,
    data_bytes: usize,
    mail_commands: usize,
    rcpt_commands: usize,
}

#[derive(Debug, Default)]
struct RemoteMxServerStats {
    sessions: usize,
    messages: usize,
    data_bytes: usize,
    mail_commands: usize,
    rcpt_commands: usize,
}

#[test]
fn orchestrates_outbound_delivery_with_mx_failover() {
    let (remote_addr, remote_handle) = spawn_mock_remote_mx(1);
    let unavailable_addr = reserve_unused_local_addr();

    let resolver = StaticResolver {
        candidates_by_domain: HashMap::from([(
            "example.net".to_string(),
            vec![
                MxCandidate::new(10, "mx-primary.verzola.test", unavailable_addr)
                    .expect("first candidate should be valid"),
                MxCandidate::new(20, "mx-secondary.verzola.test", remote_addr)
                    .expect("second candidate should be valid"),
            ],
        )]),
    };

    let (listener_addr, listener_handle) = spawn_outbound_listener(resolver);
    let (mut stream, mut reader) = connect(listener_addr);

    let banner = read_reply(&mut reader);
    assert!(banner[0].starts_with("220 "));

    send(&mut stream, "EHLO postfix.local\r\n");
    let ehlo_reply = read_reply(&mut reader);
    assert!(ehlo_reply.iter().any(|line| line.starts_with("250-")));

    send(&mut stream, "MAIL FROM:<alice@example.org>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.1.0 Sender staged for outbound relay".to_string()]
    );

    send(&mut stream, "RCPT TO:<bob@example.net>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.1.5 Recipient accepted for remote delivery".to_string()]
    );

    send(&mut stream, "DATA\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["354 End data with <CR><LF>.<CR><LF>".to_string()]
    );

    send(&mut stream, "Subject: outbound orchestration\r\n");
    send(&mut stream, "\r\n");
    send(&mut stream, "hello from postfix client\r\n");
    send(&mut stream, ".\r\n");

    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.0.0 Message accepted by remote MX".to_string()]
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Remote bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(summary.protocol_errors, 0);
    assert_eq!(summary.temporary_failures, 0);
    assert_eq!(summary.resolver_lookups, 1);
    assert_eq!(summary.mx_candidates_attempted, 2);
    assert!(summary.remote_session_established);
    assert_eq!(
        summary.selected_mx,
        Some("mx-secondary.verzola.test".to_string())
    );
    assert_eq!(
        summary.selected_recipient_domain,
        Some("example.net".to_string())
    );

    let remote_stats = join_remote(remote_handle);
    assert_eq!(remote_stats.sessions, 1);
    assert_eq!(remote_stats.messages, 1);
    assert!(remote_stats.data_bytes > 0);
    assert_eq!(remote_stats.mail_commands, 1);
    assert_eq!(remote_stats.rcpt_commands, 1);
}

#[test]
fn returns_temporary_failure_when_all_mx_candidates_are_unavailable() {
    let unavailable_addr = reserve_unused_local_addr();

    let resolver = StaticResolver {
        candidates_by_domain: HashMap::from([(
            "example.net".to_string(),
            vec![
                MxCandidate::new(10, "mx-primary.verzola.test", unavailable_addr)
                    .expect("candidate should be valid"),
            ],
        )]),
    };

    let (listener_addr, listener_handle) = spawn_outbound_listener(resolver);
    let (mut stream, mut reader) = connect(listener_addr);

    let _banner = read_reply(&mut reader);

    send(&mut stream, "EHLO postfix.local\r\n");
    let _ehlo_reply = read_reply(&mut reader);

    send(&mut stream, "MAIL FROM:<alice@example.org>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.1.0 Sender staged for outbound relay".to_string()]
    );

    send(&mut stream, "RCPT TO:<bob@example.net>\r\n");
    let rcpt_reply = read_reply(&mut reader);
    assert!(
        rcpt_reply[0].starts_with("451 4.4.0 Outbound MX temporarily unavailable:"),
        "unexpected RCPT reply: {}",
        rcpt_reply.join(" | ")
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(summary.protocol_errors, 0);
    assert_eq!(summary.temporary_failures, 1);
    assert_eq!(summary.resolver_lookups, 1);
    assert_eq!(summary.mx_candidates_attempted, 1);
    assert!(!summary.remote_session_established);
    assert_eq!(summary.selected_mx, None);
    assert_eq!(summary.selected_recipient_domain, None);
}

fn spawn_outbound_listener<R>(
    resolver: R,
) -> (
    SocketAddr,
    thread::JoinHandle<std::io::Result<OutboundSessionSummary>>,
)
where
    R: MxResolver,
{
    let config = OutboundListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "relay.verzola.test".to_string(),
        max_line_len: 4096,
    };

    let listener = OutboundListener::bind(config, resolver)
        .expect("outbound listener should bind for orchestration test");
    let address = listener.local_addr().expect("listener address must resolve");

    let handle = thread::spawn(move || listener.serve_one());
    (address, handle)
}

fn spawn_mock_remote_mx(
    expected_sessions: usize,
) -> (
    SocketAddr,
    thread::JoinHandle<std::io::Result<RemoteMxServerStats>>,
) {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("remote MX listener should bind to localhost");
    let address = listener
        .local_addr()
        .expect("remote MX listener address should resolve");

    let handle = thread::spawn(move || -> std::io::Result<RemoteMxServerStats> {
        let mut session_handles = Vec::with_capacity(expected_sessions);

        for _ in 0..expected_sessions {
            let (stream, _) = listener.accept()?;
            session_handles.push(thread::spawn(move || handle_remote_session(stream)));
        }

        let mut stats = RemoteMxServerStats::default();
        for handle in session_handles {
            let session_stats = handle
                .join()
                .map_err(|_| std::io::Error::new(ErrorKind::Other, "remote MX worker panicked"))??;
            stats.sessions += 1;
            stats.messages += session_stats.message_count;
            stats.data_bytes += session_stats.data_bytes;
            stats.mail_commands += session_stats.mail_commands;
            stats.rcpt_commands += session_stats.rcpt_commands;
        }

        Ok(stats)
    });

    (address, handle)
}

fn handle_remote_session(mut stream: TcpStream) -> std::io::Result<RemoteMxSessionStats> {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("remote MX read timeout should set");
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .expect("remote MX write timeout should set");

    write_line(&mut stream, "220 mx.secondary.verzola.test ESMTP")?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut stats = RemoteMxSessionStats::default();
    let mut reading_data = false;

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if reading_data {
            if is_data_terminator(&line) {
                reading_data = false;
                stats.message_count += 1;
                write_line(&mut stream, "250 2.0.0 Queued as ORCH1")?;
            } else {
                stats.data_bytes += line.len();
            }
            continue;
        }

        let command = line.trim_end_matches(['\r', '\n']);
        let verb = command
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();

        match verb.as_str() {
            "EHLO" | "HELO" => {
                write_line(&mut stream, "250-mx.secondary.verzola.test greets relay")?;
                write_line(&mut stream, "250 SIZE 10485760")?;
            }
            "MAIL" => {
                stats.mail_commands += 1;
                write_line(&mut stream, "250 2.1.0 Sender OK (remote mx)")?;
            }
            "RCPT" => {
                stats.rcpt_commands += 1;
                write_line(&mut stream, "250 2.1.5 Recipient OK (remote mx)")?;
            }
            "DATA" => {
                reading_data = true;
                write_line(&mut stream, "354 End data with <CR><LF>.<CR><LF>")?;
            }
            "RSET" => write_line(&mut stream, "250 2.0.0 Reset state")?,
            "NOOP" => write_line(&mut stream, "250 2.0.0 OK")?,
            "QUIT" => {
                write_line(&mut stream, "221 2.0.0 Remote bye")?;
                break;
            }
            _ => write_line(&mut stream, "502 5.5.1 Command not implemented")?,
        }
    }

    Ok(stats)
}

fn reserve_unused_local_addr() -> SocketAddr {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("ephemeral localhost listener should bind");
    let address = listener
        .local_addr()
        .expect("ephemeral localhost listener should resolve");
    drop(listener);
    address
}

fn connect(address: SocketAddr) -> (TcpStream, BufReader<TcpStream>) {
    let stream = TcpStream::connect(address).expect("test client should connect to outbound relay");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("test client read timeout should set");
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .expect("test client write timeout should set");

    let reader = BufReader::new(
        stream
            .try_clone()
            .expect("test client socket clone should succeed"),
    );

    (stream, reader)
}

fn send(stream: &mut TcpStream, command: &str) {
    stream
        .write_all(command.as_bytes())
        .expect("test client should write command bytes");
    stream.flush().expect("test client flush should succeed");
}

fn read_reply(reader: &mut BufReader<TcpStream>) -> Vec<String> {
    let mut lines = Vec::new();

    loop {
        let mut raw = String::new();
        let bytes = reader
            .read_line(&mut raw)
            .expect("test client should read relay reply");
        assert!(bytes > 0, "relay closed connection unexpectedly");

        let line = raw.trim_end_matches(['\r', '\n']).to_string();
        lines.push(line.clone());

        let line_bytes = line.as_bytes();
        if line_bytes.len() < 4 || line_bytes[3] == b' ' {
            break;
        }
    }

    lines
}

fn write_line(stream: &mut TcpStream, line: &str) -> std::io::Result<()> {
    write!(stream, "{}\r\n", line)?;
    stream.flush()
}

fn is_data_terminator(line: &str) -> bool {
    line == ".\r\n" || line == ".\n"
}

fn join_listener(
    handle: thread::JoinHandle<std::io::Result<OutboundSessionSummary>>,
) -> OutboundSessionSummary {
    handle
        .join()
        .expect("listener thread should not panic")
        .expect("listener should return summary")
}

fn join_remote(
    handle: thread::JoinHandle<std::io::Result<RemoteMxServerStats>>,
) -> RemoteMxServerStats {
    handle
        .join()
        .expect("remote thread should not panic")
        .expect("remote server should return stats")
}
