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

#[derive(Debug, Clone, Copy)]
struct RemoteBehavior {
    rcpt_reply: &'static str,
    data_command_reply: &'static str,
    data_final_reply: &'static str,
}

impl Default for RemoteBehavior {
    fn default() -> Self {
        Self {
            rcpt_reply: "250 2.1.5 Recipient OK (remote mx)",
            data_command_reply: "354 End data with <CR><LF>.<CR><LF>",
            data_final_reply: "250 2.0.0 Queued as STATUS1",
        }
    }
}

#[test]
fn maps_remote_transient_rcpt_status_to_retry_safe_defer() {
    let remote_behavior = RemoteBehavior {
        rcpt_reply: "451 4.3.0 Temporary backend issue",
        ..RemoteBehavior::default()
    };
    let (remote_addr, remote_handle) = spawn_mock_remote_mx(1, remote_behavior);

    let resolver = StaticResolver {
        candidates_by_domain: HashMap::from([(
            "example.net".to_string(),
            vec![
                MxCandidate::new(10, "mx-status.verzola.test", remote_addr)
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
    assert_eq!(
        read_reply(&mut reader),
        vec![
            "451 4.4.0 Delivery deferred for retry (stage=rcpt, class=remote-transient, upstream=451)"
                .to_string()
        ]
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Remote bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(summary.temporary_failures, 1);
    assert_eq!(summary.resolver_lookups, 1);
    assert_eq!(summary.mx_candidates_attempted, 1);
    assert!(summary.remote_session_established);
    assert_eq!(summary.selected_mx, Some("mx-status.verzola.test".to_string()));
    assert_eq!(
        summary.selected_recipient_domain,
        Some("example.net".to_string())
    );

    join_remote(remote_handle);
}

#[test]
fn maps_remote_permanent_data_status_to_retry_safe_defer() {
    let remote_behavior = RemoteBehavior {
        data_final_reply: "554 5.6.0 Content rejected by remote policy",
        ..RemoteBehavior::default()
    };
    let (remote_addr, remote_handle) = spawn_mock_remote_mx(1, remote_behavior);

    let resolver = StaticResolver {
        candidates_by_domain: HashMap::from([(
            "example.net".to_string(),
            vec![
                MxCandidate::new(10, "mx-status.verzola.test", remote_addr)
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
    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.1.5 Recipient accepted for remote delivery".to_string()]
    );

    send(&mut stream, "DATA\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["354 End data with <CR><LF>.<CR><LF>".to_string()]
    );

    send(&mut stream, "Subject: status contract\r\n");
    send(&mut stream, "\r\n");
    send(&mut stream, "hello from postfix client\r\n");
    send(&mut stream, ".\r\n");

    assert_eq!(
        read_reply(&mut reader),
        vec![
            "451 4.4.0 Delivery deferred for retry (stage=data-final, class=remote-permanent, upstream=554)"
                .to_string()
        ]
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Remote bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(summary.temporary_failures, 1);
    assert_eq!(summary.resolver_lookups, 1);
    assert_eq!(summary.mx_candidates_attempted, 1);
    assert!(summary.remote_session_established);
    assert_eq!(summary.selected_mx, Some("mx-status.verzola.test".to_string()));
    assert_eq!(
        summary.selected_recipient_domain,
        Some("example.net".to_string())
    );

    join_remote(remote_handle);
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
        .expect("outbound listener should bind for status-contract test");
    let address = listener.local_addr().expect("listener address must resolve");

    let handle = thread::spawn(move || listener.serve_one());
    (address, handle)
}

fn spawn_mock_remote_mx(
    expected_sessions: usize,
    behavior: RemoteBehavior,
) -> (SocketAddr, thread::JoinHandle<std::io::Result<()>>) {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("remote MX listener should bind to localhost");
    let address = listener
        .local_addr()
        .expect("remote MX listener address should resolve");

    let handle = thread::spawn(move || -> std::io::Result<()> {
        let mut session_handles = Vec::with_capacity(expected_sessions);

        for _ in 0..expected_sessions {
            let (stream, _) = listener.accept()?;
            session_handles.push(thread::spawn(move || handle_remote_session(stream, behavior)));
        }

        for handle in session_handles {
            handle
                .join()
                .map_err(|_| std::io::Error::new(ErrorKind::Other, "remote MX worker panicked"))??;
        }

        Ok(())
    });

    (address, handle)
}

fn handle_remote_session(mut stream: TcpStream, behavior: RemoteBehavior) -> std::io::Result<()> {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("remote MX read timeout should set");
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .expect("remote MX write timeout should set");

    write_line(&mut stream, "220 mx.status.verzola.test ESMTP")?;

    let mut reader = BufReader::new(stream.try_clone()?);
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
                write_line(&mut stream, behavior.data_final_reply)?;
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
                write_line(&mut stream, "250-mx.status.verzola.test greets relay")?;
                write_line(&mut stream, "250 SIZE 10485760")?;
            }
            "MAIL" => write_line(&mut stream, "250 2.1.0 Sender OK (remote mx)")?,
            "RCPT" => write_line(&mut stream, behavior.rcpt_reply)?,
            "DATA" => {
                write_line(&mut stream, behavior.data_command_reply)?;
                if behavior.data_command_reply.starts_with("354 ") {
                    reading_data = true;
                }
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

    Ok(())
}

fn connect(address: SocketAddr) -> (TcpStream, BufReader<TcpStream>) {
    let stream =
        TcpStream::connect(address).expect("test client should connect to outbound relay listener");
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

fn join_remote(handle: thread::JoinHandle<std::io::Result<()>>) {
    handle
        .join()
        .expect("remote thread should not panic")
        .expect("remote server should return success")
}
