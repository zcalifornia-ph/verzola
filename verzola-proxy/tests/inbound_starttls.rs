use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

use verzola_proxy::inbound::{
    InboundListener, ListenerConfig, NoopTlsUpgrader, SessionSummary, TlsUpgradeError, TlsUpgrader,
};

#[derive(Debug, Clone, Copy)]
struct FailingTlsUpgrader;

impl TlsUpgrader for FailingTlsUpgrader {
    fn upgrade(&self, _stream: &mut TcpStream) -> Result<(), TlsUpgradeError> {
        Err(TlsUpgradeError::Temporary(
            "simulated handshake failure".to_string(),
        ))
    }
}

#[test]
fn starttls_success_requires_ehlo_reset() {
    let (address, handle) = spawn_server(NoopTlsUpgrader);
    let (mut stream, mut reader) = connect(address);

    let banner = read_reply(&mut reader);
    assert!(banner[0].starts_with("220 "));

    send(&mut stream, "EHLO client.example\r\n");
    let ehlo_reply = read_reply(&mut reader);
    assert!(ehlo_reply.iter().any(|line| line == "250-STARTTLS"));

    send(&mut stream, "STARTTLS\r\n");
    let starttls_reply = read_reply(&mut reader);
    assert_eq!(starttls_reply, vec!["220 Ready to start TLS".to_string()]);

    send(&mut stream, "MAIL FROM:<alice@example.com>\r\n");
    let needs_ehlo_reply = read_reply(&mut reader);
    assert_eq!(
        needs_ehlo_reply,
        vec!["503 5.5.1 Send EHLO after STARTTLS".to_string()]
    );

    send(&mut stream, "EHLO client.example\r\n");
    let ehlo_after_tls = read_reply(&mut reader);
    assert!(!ehlo_after_tls.iter().any(|line| line == "250-STARTTLS"));

    send(&mut stream, "MAIL FROM:<alice@example.com>\r\n");
    let mail_reply = read_reply(&mut reader);
    assert_eq!(mail_reply, vec!["250 2.1.0 Sender OK".to_string()]);

    send(&mut stream, "QUIT\r\n");
    let quit_reply = read_reply(&mut reader);
    assert_eq!(quit_reply, vec!["221 2.0.0 Bye".to_string()]);

    let summary = join_server(handle);
    assert!(summary.tls_negotiated);
}

#[test]
fn starttls_failure_maps_to_454() {
    let (address, handle) = spawn_server(FailingTlsUpgrader);
    let (mut stream, mut reader) = connect(address);

    let _ = read_reply(&mut reader);

    send(&mut stream, "EHLO failing-client\r\n");
    let _ = read_reply(&mut reader);

    send(&mut stream, "STARTTLS\r\n");
    let ready_reply = read_reply(&mut reader);
    assert_eq!(ready_reply, vec!["220 Ready to start TLS".to_string()]);

    let failure_reply = read_reply(&mut reader);
    assert_eq!(
        failure_reply,
        vec![
            "454 4.7.0 TLS not available due to temporary reason: simulated handshake failure"
                .to_string()
        ]
    );

    send(&mut stream, "QUIT\r\n");
    let _ = read_reply(&mut reader);

    let summary = join_server(handle);
    assert!(!summary.tls_negotiated);
    assert!(summary.protocol_errors >= 1);
}

#[test]
fn starttls_before_ehlo_is_rejected() {
    let (address, handle) = spawn_server(NoopTlsUpgrader);
    let (mut stream, mut reader) = connect(address);

    let _ = read_reply(&mut reader);

    send(&mut stream, "STARTTLS\r\n");
    let reply = read_reply(&mut reader);
    assert_eq!(
        reply,
        vec!["503 5.5.1 Send EHLO before STARTTLS".to_string()]
    );

    send(&mut stream, "QUIT\r\n");
    let _ = read_reply(&mut reader);

    let summary = join_server(handle);
    assert!(!summary.tls_negotiated);
    assert!(summary.protocol_errors >= 1);
}

fn spawn_server<U>(tls_upgrader: U) -> (SocketAddr, thread::JoinHandle<std::io::Result<SessionSummary>>)
where
    U: TlsUpgrader,
{
    let config = ListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "mx.verzola.test".to_string(),
        advertise_starttls: true,
        max_line_len: 4096,
    };

    let listener = InboundListener::bind(config, tls_upgrader)
        .expect("listener must bind for integration test");
    let address = listener.local_addr().expect("listener address must resolve");

    let handle = thread::spawn(move || listener.serve_one());
    (address, handle)
}

fn connect(address: SocketAddr) -> (TcpStream, BufReader<TcpStream>) {
    let stream = TcpStream::connect(address).expect("client must connect to listener");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("test socket should accept read timeout");
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("test socket should accept write timeout");
    let reader = BufReader::new(
        stream
            .try_clone()
            .expect("test socket clone for reader should succeed"),
    );
    (stream, reader)
}

fn send(stream: &mut TcpStream, command: &str) {
    stream
        .write_all(command.as_bytes())
        .expect("test command write should succeed");
    stream.flush().expect("test command flush should succeed");
}

fn read_reply(reader: &mut BufReader<TcpStream>) -> Vec<String> {
    let mut lines = Vec::new();
    loop {
        let mut raw = String::new();
        let bytes = reader
            .read_line(&mut raw)
            .expect("test should read SMTP server reply");
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

fn join_server(handle: thread::JoinHandle<std::io::Result<SessionSummary>>) -> SessionSummary {
    handle
        .join()
        .expect("server thread should not panic")
        .expect("server must return session summary")
}
