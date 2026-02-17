use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;

use verzola_proxy::inbound::{
    InboundListener, InboundTlsPolicy, ListenerConfig, NoopTlsUpgrader, SessionSummary,
    TlsUpgradeError, TlsUpgrader,
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
fn opportunistic_policy_allows_plaintext_mail_flow() {
    let (address, handle) = spawn_server(InboundTlsPolicy::Opportunistic, NoopTlsUpgrader, true);
    let (mut stream, mut reader) = connect(address);

    let _banner = read_reply(&mut reader);

    send(&mut stream, "EHLO opportunistic.example\r\n");
    let ehlo_reply = read_reply(&mut reader);
    assert!(ehlo_reply.iter().any(|line| line == "250-STARTTLS"));

    send(&mut stream, "MAIL FROM:<alice@example.com>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.1.0 Sender OK".to_string()]
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Bye".to_string()]);

    let summary = join_server(handle);
    assert_eq!(summary.inbound_tls_policy, InboundTlsPolicy::Opportunistic);
    assert!(!summary.tls_negotiated);
    assert_eq!(summary.telemetry.starttls_attempts, 0);
    assert_eq!(summary.telemetry.tls_upgrade_failures, 0);
    assert_eq!(summary.telemetry.require_tls_rejections, 0);
}

#[test]
fn require_tls_policy_rejects_plaintext_until_starttls() {
    let (address, handle) = spawn_server(InboundTlsPolicy::RequireTls, NoopTlsUpgrader, true);
    let (mut stream, mut reader) = connect(address);

    let _banner = read_reply(&mut reader);

    send(&mut stream, "EHLO secure.example\r\n");
    let ehlo_reply = read_reply(&mut reader);
    assert!(ehlo_reply.iter().any(|line| line == "250-STARTTLS"));

    send(&mut stream, "MAIL FROM:<alice@example.com>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["530 5.7.0 Must issue STARTTLS first".to_string()]
    );

    send(&mut stream, "RCPT TO:<bob@example.net>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["530 5.7.0 Must issue STARTTLS first".to_string()]
    );

    send(&mut stream, "DATA\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["530 5.7.0 Must issue STARTTLS first".to_string()]
    );

    send(&mut stream, "STARTTLS\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["220 Ready to start TLS".to_string()]
    );

    send(&mut stream, "EHLO secure.example\r\n");
    let ehlo_after_tls = read_reply(&mut reader);
    assert!(!ehlo_after_tls.iter().any(|line| line == "250-STARTTLS"));

    send(&mut stream, "MAIL FROM:<alice@example.com>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["250 2.1.0 Sender OK".to_string()]
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Bye".to_string()]);

    let summary = join_server(handle);
    assert_eq!(summary.inbound_tls_policy, InboundTlsPolicy::RequireTls);
    assert!(summary.tls_negotiated);
    assert_eq!(summary.telemetry.starttls_attempts, 1);
    assert_eq!(summary.telemetry.tls_upgrade_failures, 0);
    assert_eq!(summary.telemetry.require_tls_rejections, 3);
}

#[test]
fn require_tls_listener_config_requires_starttls_advertisement() {
    let config = ListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "mx.verzola.test".to_string(),
        advertise_starttls: false,
        inbound_tls_policy: InboundTlsPolicy::RequireTls,
        max_line_len: 4096,
        postfix_upstream_addr: None,
    };

    let error = match InboundListener::bind(config, NoopTlsUpgrader) {
        Ok(_) => panic!("require-tls without STARTTLS advertisement must fail"),
        Err(error) => error,
    };

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert!(
        error
            .to_string()
            .contains("inbound_tls_policy=require-tls requires advertise_starttls=true"),
        "unexpected error message: {}",
        error
    );
}

#[test]
fn telemetry_tracks_tls_failures_and_policy_rejections() {
    let (address, handle) = spawn_server(InboundTlsPolicy::RequireTls, FailingTlsUpgrader, true);
    let (mut stream, mut reader) = connect(address);

    let _banner = read_reply(&mut reader);

    send(&mut stream, "EHLO failing.example\r\n");
    let _ = read_reply(&mut reader);

    send(&mut stream, "STARTTLS\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["220 Ready to start TLS".to_string()]
    );
    assert_eq!(
        read_reply(&mut reader),
        vec![
            "454 4.7.0 TLS not available due to temporary reason: simulated handshake failure"
                .to_string()
        ]
    );

    send(&mut stream, "MAIL FROM:<alice@example.com>\r\n");
    assert_eq!(
        read_reply(&mut reader),
        vec!["530 5.7.0 Must issue STARTTLS first".to_string()]
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Bye".to_string()]);

    let summary = join_server(handle);
    assert_eq!(summary.inbound_tls_policy, InboundTlsPolicy::RequireTls);
    assert!(!summary.tls_negotiated);
    assert_eq!(summary.telemetry.starttls_attempts, 1);
    assert_eq!(summary.telemetry.tls_upgrade_failures, 1);
    assert_eq!(summary.telemetry.require_tls_rejections, 1);
}

fn spawn_server<U>(
    policy: InboundTlsPolicy,
    tls_upgrader: U,
    advertise_starttls: bool,
) -> (
    SocketAddr,
    thread::JoinHandle<std::io::Result<SessionSummary>>,
)
where
    U: TlsUpgrader,
{
    let config = ListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "mx.verzola.test".to_string(),
        advertise_starttls,
        inbound_tls_policy: policy,
        max_line_len: 4096,
        postfix_upstream_addr: None,
    };

    let listener =
        InboundListener::bind(config, tls_upgrader).expect("listener must bind for integration test");
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
