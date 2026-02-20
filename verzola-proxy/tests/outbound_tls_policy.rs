use std::collections::HashMap;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use verzola_proxy::outbound::{
    MxCandidate, MxResolutionError, MxResolver, OutboundDomainTlsPolicy, OutboundListener,
    OutboundListenerConfig, OutboundSessionSummary, OutboundTlsPolicy,
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
    advertise_starttls: bool,
    starttls_reply: &'static str,
    rcpt_reply: &'static str,
}

impl Default for RemoteBehavior {
    fn default() -> Self {
        Self {
            advertise_starttls: false,
            starttls_reply: "220 2.0.0 Ready to start TLS",
            rcpt_reply: "250 2.1.5 Recipient OK (remote mx)",
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct RemoteSessionStats {
    starttls_commands: usize,
    mail_commands: usize,
    rcpt_commands: usize,
}

#[derive(Debug, Default)]
struct RemoteServerStats {
    sessions: usize,
    starttls_commands: usize,
    mail_commands: usize,
    rcpt_commands: usize,
}

#[test]
fn opportunistic_policy_allows_plaintext_when_starttls_is_unavailable() {
    let behavior = RemoteBehavior::default();
    let (remote_addr, remote_handle) = spawn_mock_remote_mx(1, behavior);

    let resolver = resolver_for_domain("example.net", remote_addr, "mx-policy.verzola.test");
    let config = OutboundListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "relay.verzola.test".to_string(),
        outbound_tls_policy: OutboundTlsPolicy::Opportunistic,
        per_domain_tls_policies: Vec::new(),
        max_line_len: 4096,
    };

    let (listener_addr, listener_handle) = spawn_outbound_listener(resolver, config);
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

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Remote bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(
        summary.effective_tls_policy,
        Some(OutboundTlsPolicy::Opportunistic)
    );
    assert!(!summary.tls_negotiated);
    assert_eq!(summary.opportunistic_tls_fallbacks, 0);
    assert_eq!(summary.policy_deferred_failures, 0);
    assert_eq!(summary.temporary_failures, 0);
    assert!(summary.remote_session_established);

    let remote_stats = join_remote(remote_handle);
    assert_eq!(remote_stats.sessions, 1);
    assert_eq!(remote_stats.starttls_commands, 0);
    assert_eq!(remote_stats.mail_commands, 1);
    assert_eq!(remote_stats.rcpt_commands, 1);
}

#[test]
fn opportunistic_policy_falls_back_when_starttls_negotiation_fails() {
    let behavior = RemoteBehavior {
        advertise_starttls: true,
        starttls_reply: "454 4.7.0 TLS unavailable",
        ..RemoteBehavior::default()
    };
    let (remote_addr, remote_handle) = spawn_mock_remote_mx(2, behavior);

    let resolver = resolver_for_domain("example.net", remote_addr, "mx-policy.verzola.test");
    let config = OutboundListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "relay.verzola.test".to_string(),
        outbound_tls_policy: OutboundTlsPolicy::Opportunistic,
        per_domain_tls_policies: Vec::new(),
        max_line_len: 4096,
    };

    let (listener_addr, listener_handle) = spawn_outbound_listener(resolver, config);
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

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Remote bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(
        summary.effective_tls_policy,
        Some(OutboundTlsPolicy::Opportunistic)
    );
    assert!(!summary.tls_negotiated);
    assert_eq!(summary.opportunistic_tls_fallbacks, 1);
    assert_eq!(summary.policy_deferred_failures, 0);
    assert_eq!(summary.temporary_failures, 0);
    assert!(summary.remote_session_established);

    let remote_stats = join_remote(remote_handle);
    assert_eq!(remote_stats.sessions, 2);
    assert_eq!(remote_stats.starttls_commands, 1);
    assert_eq!(remote_stats.mail_commands, 1);
    assert_eq!(remote_stats.rcpt_commands, 1);
}

#[test]
fn require_tls_policy_defers_when_peer_does_not_advertise_starttls() {
    let behavior = RemoteBehavior::default();
    let (remote_addr, remote_handle) = spawn_mock_remote_mx(1, behavior);

    let resolver = resolver_for_domain("example.net", remote_addr, "mx-policy.verzola.test");
    let config = OutboundListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "relay.verzola.test".to_string(),
        outbound_tls_policy: OutboundTlsPolicy::RequireTls,
        per_domain_tls_policies: Vec::new(),
        max_line_len: 4096,
    };

    let (listener_addr, listener_handle) = spawn_outbound_listener(resolver, config);
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
        rcpt_reply[0].starts_with("451 4.7.5 Outbound TLS policy defer:"),
        "unexpected RCPT reply: {}",
        rcpt_reply.join(" | ")
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(
        summary.effective_tls_policy,
        Some(OutboundTlsPolicy::RequireTls)
    );
    assert!(!summary.tls_negotiated);
    assert_eq!(summary.opportunistic_tls_fallbacks, 0);
    assert_eq!(summary.policy_deferred_failures, 1);
    assert_eq!(summary.temporary_failures, 1);
    assert!(!summary.remote_session_established);
    assert_eq!(summary.selected_mx, None);

    let remote_stats = join_remote(remote_handle);
    assert_eq!(remote_stats.sessions, 1);
    assert_eq!(remote_stats.starttls_commands, 0);
    assert_eq!(remote_stats.mail_commands, 0);
    assert_eq!(remote_stats.rcpt_commands, 0);
}

#[test]
fn per_domain_rule_overrides_global_policy_for_stricter_domain() {
    let behavior = RemoteBehavior::default();
    let (remote_addr, remote_handle) = spawn_mock_remote_mx(1, behavior);

    let resolver = resolver_for_domain("example.net", remote_addr, "mx-policy.verzola.test");
    let config = OutboundListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "relay.verzola.test".to_string(),
        outbound_tls_policy: OutboundTlsPolicy::Opportunistic,
        per_domain_tls_policies: vec![
            OutboundDomainTlsPolicy::new("example.net", OutboundTlsPolicy::RequireTls)
                .expect("domain policy should be valid"),
        ],
        max_line_len: 4096,
    };

    let (listener_addr, listener_handle) = spawn_outbound_listener(resolver, config);
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
        rcpt_reply[0].starts_with("451 4.7.5 Outbound TLS policy defer:"),
        "unexpected RCPT reply: {}",
        rcpt_reply.join(" | ")
    );

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(
        summary.effective_tls_policy,
        Some(OutboundTlsPolicy::RequireTls)
    );
    assert_eq!(summary.policy_deferred_failures, 1);
    assert_eq!(summary.temporary_failures, 1);
    assert!(!summary.remote_session_established);

    let remote_stats = join_remote(remote_handle);
    assert_eq!(remote_stats.sessions, 1);
}

#[test]
fn per_domain_rule_can_relax_global_require_tls_policy() {
    let behavior = RemoteBehavior::default();
    let (remote_addr, remote_handle) = spawn_mock_remote_mx(1, behavior);

    let resolver = resolver_for_domain("example.net", remote_addr, "mx-policy.verzola.test");
    let config = OutboundListenerConfig {
        bind_addr: "127.0.0.1:0"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "relay.verzola.test".to_string(),
        outbound_tls_policy: OutboundTlsPolicy::RequireTls,
        per_domain_tls_policies: vec![
            OutboundDomainTlsPolicy::new("example.net", OutboundTlsPolicy::Opportunistic)
                .expect("domain policy should be valid"),
        ],
        max_line_len: 4096,
    };

    let (listener_addr, listener_handle) = spawn_outbound_listener(resolver, config);
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

    send(&mut stream, "QUIT\r\n");
    assert_eq!(read_reply(&mut reader), vec!["221 2.0.0 Remote bye".to_string()]);

    let summary = join_listener(listener_handle);
    assert_eq!(
        summary.effective_tls_policy,
        Some(OutboundTlsPolicy::Opportunistic)
    );
    assert_eq!(summary.policy_deferred_failures, 0);
    assert_eq!(summary.temporary_failures, 0);
    assert!(summary.remote_session_established);

    let remote_stats = join_remote(remote_handle);
    assert_eq!(remote_stats.sessions, 1);
    assert_eq!(remote_stats.mail_commands, 1);
    assert_eq!(remote_stats.rcpt_commands, 1);
}

#[test]
fn config_validation_rejects_duplicate_domain_policy_rules() {
    let config = OutboundListenerConfig {
        bind_addr: "127.0.0.1:10025"
            .parse()
            .expect("hard-coded socket address must parse"),
        banner_host: "relay.verzola.test".to_string(),
        outbound_tls_policy: OutboundTlsPolicy::Opportunistic,
        per_domain_tls_policies: vec![
            OutboundDomainTlsPolicy::new("Example.NET", OutboundTlsPolicy::RequireTls)
                .expect("first domain policy should be valid"),
            OutboundDomainTlsPolicy::new("example.net", OutboundTlsPolicy::Opportunistic)
                .expect("second domain policy should be valid"),
        ],
        max_line_len: 4096,
    };

    let error = config
        .validate()
        .expect_err("duplicate normalized domain policy rules should fail");

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert!(
        error
            .to_string()
            .contains("per_domain_tls_policies contains duplicate domain rule"),
        "unexpected validation error: {}",
        error
    );
}

fn resolver_for_domain(
    domain: &str,
    remote_addr: SocketAddr,
    exchange: &str,
) -> StaticResolver {
    StaticResolver {
        candidates_by_domain: HashMap::from([(
            domain.to_string(),
            vec![
                MxCandidate::new(10, exchange, remote_addr).expect("candidate should be valid"),
            ],
        )]),
    }
}

fn spawn_outbound_listener<R>(
    resolver: R,
    config: OutboundListenerConfig,
) -> (
    SocketAddr,
    thread::JoinHandle<std::io::Result<OutboundSessionSummary>>,
)
where
    R: MxResolver,
{
    let listener = OutboundListener::bind(config, resolver)
        .expect("outbound listener should bind for policy test");
    let address = listener.local_addr().expect("listener address must resolve");

    let handle = thread::spawn(move || listener.serve_one());
    (address, handle)
}

fn spawn_mock_remote_mx(
    expected_sessions: usize,
    behavior: RemoteBehavior,
) -> (
    SocketAddr,
    thread::JoinHandle<std::io::Result<RemoteServerStats>>,
) {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("remote MX listener should bind to localhost");
    let address = listener
        .local_addr()
        .expect("remote MX listener address should resolve");

    let handle = thread::spawn(move || -> std::io::Result<RemoteServerStats> {
        let mut session_handles = Vec::with_capacity(expected_sessions);

        for _ in 0..expected_sessions {
            let (stream, _) = listener.accept()?;
            session_handles.push(thread::spawn(move || handle_remote_session(stream, behavior)));
        }

        let mut stats = RemoteServerStats::default();
        for handle in session_handles {
            let session_stats = handle
                .join()
                .map_err(|_| std::io::Error::new(ErrorKind::Other, "remote MX worker panicked"))??;

            stats.sessions += 1;
            stats.starttls_commands += session_stats.starttls_commands;
            stats.mail_commands += session_stats.mail_commands;
            stats.rcpt_commands += session_stats.rcpt_commands;
        }

        Ok(stats)
    });

    (address, handle)
}

fn handle_remote_session(
    mut stream: TcpStream,
    behavior: RemoteBehavior,
) -> std::io::Result<RemoteSessionStats> {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("remote MX read timeout should set");
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .expect("remote MX write timeout should set");

    write_line(&mut stream, "220 mx.policy.verzola.test ESMTP")?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut stats = RemoteSessionStats::default();

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        let command = line.trim_end_matches(['\r', '\n']);
        let verb = command
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();

        match verb.as_str() {
            "EHLO" | "HELO" => {
                write_line(&mut stream, "250-mx.policy.verzola.test greets relay")?;
                if behavior.advertise_starttls {
                    write_line(&mut stream, "250-STARTTLS")?;
                }
                write_line(&mut stream, "250 SIZE 10485760")?;
            }
            "STARTTLS" => {
                stats.starttls_commands += 1;
                write_line(&mut stream, behavior.starttls_reply)?;
            }
            "MAIL" => {
                stats.mail_commands += 1;
                write_line(&mut stream, "250 2.1.0 Sender OK (remote mx)")?;
            }
            "RCPT" => {
                stats.rcpt_commands += 1;
                write_line(&mut stream, behavior.rcpt_reply)?;
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

fn join_listener(
    handle: thread::JoinHandle<std::io::Result<OutboundSessionSummary>>,
) -> OutboundSessionSummary {
    handle
        .join()
        .expect("listener thread should not panic")
        .expect("listener should return summary")
}

fn join_remote(
    handle: thread::JoinHandle<std::io::Result<RemoteServerStats>>,
) -> RemoteServerStats {
    handle
        .join()
        .expect("remote thread should not panic")
        .expect("remote server should return stats")
}
