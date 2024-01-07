use std::net::SocketAddr;

use common::config::SERVER_PORT;

pub fn default_server_addr() -> SocketAddr {
    local_ip_address::local_ip()
        .map(|ip| format!("{}:{}", ip, SERVER_PORT))
        .unwrap()
        .parse()
        .unwrap()
}
