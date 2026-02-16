use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use verzola_proxy::inbound::{InboundListener, ListenerConfig, NoopTlsUpgrader, SessionSummary};

#[derive(Debug, Default, Clone, Copy)]
struct PostfixSessionStats {
    data_bytes: usize,
    message_count: usize,
}

#[derive(Debug, Default)]
struct PostfixServerStats {
    sessions: usize,
    messages: usize,
    max_session_data_bytes: usize,
    total_data_bytes: usize,
}

#[test]
fn relays_large_data_block_to_postfix_loopback() {
    let (postfix_addr, postfix_handle) = spawn_mock_postfix(1);
    let (listener_addr, listener_handle) = spawn_relay_listener(postfix_addr, 1);

    let (mut stream, mut reader) = connect(listener_addr);
    let banner = read_reply(&mut reader);
    assert!(banner[0].starts_with("220 "));

    send(&mut stream, "EHLO sender.example\r\n");
    let ehlo_reply = read_reply(&mut reader);
    assert!(ehlo_reply.iter().any(|line| line.starts_with("250-STARTTLS")));

    send(&mut stream, "MAIL FROM:<alice@example.com>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.1.0 Sender OK (loopback postfix)".to_string()]
    );

    send(&mut stream, "RCPT TO:<bob@example.net>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.1.5 Recipient OK (loopback postfix)".to_string()]
    );

    send(&mut stream, "DATA\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["354 End data with <CR><LF>.<CR><LF>".to_string()]
    );

    let large_line = format!("{}\r\n", "a".repeat(2048));
    for _ in 0..300 {
        send(&mut stream, &large_line);
    }
    send(&mut stream, ".\r\n");

    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.0.0 Queued as LOOPBACK".to_string()]
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Bye".to_string()]);

    let session_summaries = join_listener(listener_handle);
    assert_eq!(session_summaries.len(), 1);
    assert_eq!(session_summaries[0].protocol_errors, 0);

    let postfix_stats = join_postfix(postfix_handle);
    assert_eq!(postfix_stats.sessions, 1);
    assert_eq!(postfix_stats.messages, 1);
    assert!(
        postfix_stats.max_session_data_bytes >= 614_400,
        "expected >= 614400 bytes relayed, got {}",
        postfix_stats.max_session_data_bytes
    );
}

#[test]
fn relays_concurrent_sessions_without_cross_talk() {
    let (postfix_addr, postfix_handle) = spawn_mock_postfix(2);
    let (listener_addr, listener_handle) = spawn_relay_listener(postfix_addr, 2);

    let barrier = Arc::new(Barrier::new(3));
    let mut client_handles = Vec::new();

    for client_id in 0..2 {
        let address = listener_addr;
        let start_gate = Arc::clone(&barrier);
        client_handles.push(thread::spawn(move || {
            run_client_session(address, client_id, start_gate)
        }));
    }

    barrier.wait();

    for handle in client_handles {
        handle
            .join()
            .expect("client thread should not panic")
            .expect("client session should complete");
    }

    let session_summaries = join_listener(listener_handle);
    assert_eq!(session_summaries.len(), 2);
    assert!(
        session_summaries
            .iter()
            .all(|summary| summary.protocol_errors == 0)
    );

    let postfix_stats = join_postfix(postfix_handle);
    assert_eq!(postfix_stats.sessions, 2);
    assert_eq!(postfix_stats.messages, 2);
    assert!(postfix_stats.total_data_bytes >= 262_144);
}

fn run_client_session(
    address: SocketAddr,
    client_id: usize,
    start_gate: Arc<Barrier>,
) -> std::io::Result<()> {
    let (mut stream, mut reader) = connect(address);

    let _banner = read_reply(&mut reader);

    send(&mut stream, &format!("EHLO client{}.example\r\n", client_id));
    let _ehlo_reply = read_reply(&mut reader);

    send(
        &mut stream,
        &format!("MAIL FROM:<sender{}@example.com>\r\n", client_id),
    );
    let mail_reply = read_reply(&mut reader);
    if !mail_reply[0].starts_with("250 ") {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            format!("unexpected MAIL reply: {}", mail_reply.join(" | ")),
        ));
    }

    send(
        &mut stream,
        &format!("RCPT TO:<recipient{}@example.net>\r\n", client_id),
    );
    let rcpt_reply = read_reply(&mut reader);
    if !rcpt_reply[0].starts_with("250 ") {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            format!("unexpected RCPT reply: {}", rcpt_reply.join(" | ")),
        ));
    }

    send(&mut stream, "DATA\r\n");
    let data_reply = read_reply(&mut reader);
    if data_reply != vec!["354 End data with <CR><LF>.<CR><LF>".to_string()] {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            format!("unexpected DATA reply: {}", data_reply.join(" | ")),
        ));
    }

    start_gate.wait();

    for chunk_index in 0..128 {
        let line = format!(
            "client={} chunk={} payload={}\r\n",
            client_id,
            chunk_index,
            "x".repeat(1024)
        );
        send(&mut stream, &line);
        if chunk_index % 16 == 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }
    send(&mut stream, ".\r\n");

    let final_reply = read_reply(&mut reader);
    if !final_reply[0].starts_with("250 ") {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            format!("unexpected final DATA reply: {}", final_reply.join(" | ")),
        ));
    }

    send(&mut stream, "QUIT\r\n");
    let quit_reply = read_reply(&mut reader);
    if !quit_reply[0].starts_with("221 ") {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            format!("unexpected QUIT reply: {}", quit_reply.join(" | ")),
        ));
    }

    Ok(())
}

fn spawn_relay_listener(
    postfix_addr: SocketAddr,
    session_count: usize,
) -> (
    SocketAddr,
    thread::JoinHandle<std::io::Result<Vec<SessionSummary>>>,
) {
    let config = ListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "mx.verzola.test".to_string(),
        advertise_starttls: true,
        max_line_len: 4096,
        postfix_upstream_addr: Some(postfix_addr),
    };

    let listener = InboundListener::bind(config, NoopTlsUpgrader)
        .expect("listener should bind for forwarder integration test");
    let address = listener.local_addr().expect("listener address must resolve");
    let handle = thread::spawn(move || listener.serve_n(session_count));

    (address, handle)
}

fn spawn_mock_postfix(
    expected_sessions: usize,
) -> (
    SocketAddr,
    thread::JoinHandle<std::io::Result<PostfixServerStats>>,
) {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("mock postfix listener should bind to localhost");
    let address = listener
        .local_addr()
        .expect("mock postfix listener address should resolve");

    let handle = thread::spawn(move || -> std::io::Result<PostfixServerStats> {
        let mut session_handles = Vec::with_capacity(expected_sessions);

        for _ in 0..expected_sessions {
            let (stream, _) = listener.accept()?;
            session_handles.push(thread::spawn(move || handle_postfix_session(stream)));
        }

        let mut stats = PostfixServerStats::default();
        for handle in session_handles {
            let session_stats = handle
                .join()
                .map_err(|_| std::io::Error::new(ErrorKind::Other, "postfix worker panicked"))??;
            stats.sessions += 1;
            stats.messages += session_stats.message_count;
            stats.total_data_bytes += session_stats.data_bytes;
            stats.max_session_data_bytes = stats.max_session_data_bytes.max(session_stats.data_bytes);
        }

        Ok(stats)
    });

    (address, handle)
}

fn handle_postfix_session(mut stream: TcpStream) -> std::io::Result<PostfixSessionStats> {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("mock postfix read timeout should set");
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .expect("mock postfix write timeout should set");

    write_line(&mut stream, "220 postfix.loopback ESMTP")?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut stats = PostfixSessionStats::default();
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
                write_line(&mut stream, "250 2.0.0 Queued as LOOPBACK")?;
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
                write_line(&mut stream, "250-postfix.loopback greets relay")?;
                write_line(&mut stream, "250 SIZE 10485760")?;
            }
            "MAIL" => write_line(&mut stream, "250 2.1.0 Sender OK (loopback postfix)")?,
            "RCPT" => write_line(&mut stream, "250 2.1.5 Recipient OK (loopback postfix)")?,
            "DATA" => {
                reading_data = true;
                write_line(&mut stream, "354 End data with <CR><LF>.<CR><LF>")?;
            }
            "RSET" => write_line(&mut stream, "250 2.0.0 Reset state")?,
            "NOOP" => write_line(&mut stream, "250 2.0.0 OK")?,
            "QUIT" => {
                write_line(&mut stream, "221 2.0.0 Bye")?;
                break;
            }
            _ => write_line(&mut stream, "502 5.5.1 Command not implemented")?,
        }
    }

    Ok(stats)
}

fn connect(address: SocketAddr) -> (TcpStream, BufReader<TcpStream>) {
    let stream = TcpStream::connect(address).expect("client should connect to test listener");
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
            .expect("test client should read server reply");
        assert!(bytes > 0, "server closed connection unexpectedly");

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
    handle: thread::JoinHandle<std::io::Result<Vec<SessionSummary>>>,
) -> Vec<SessionSummary> {
    handle
        .join()
        .expect("listener thread should not panic")
        .expect("listener should return session summaries")
}

fn join_postfix(
    handle: thread::JoinHandle<std::io::Result<PostfixServerStats>>,
) -> PostfixServerStats {
    handle
        .join()
        .expect("postfix thread should not panic")
        .expect("mock postfix should return stats")
}
