use std::net::SocketAddr;

use verzola_proxy::inbound::{InboundListener, ListenerConfig, NoopTlsUpgrader};

fn main() -> std::io::Result<()> {
    let bind_addr: SocketAddr = "127.0.0.1:2525"
        .parse()
        .expect("hard-coded socket address must be valid");

    let config = ListenerConfig {
        bind_addr,
        banner_host: "localhost".to_string(),
        advertise_starttls: true,
        max_line_len: 4096,
    };

    let listener = InboundListener::bind(config, NoopTlsUpgrader)?;
    listener.serve_one()?;

    Ok(())
}
