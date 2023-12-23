use std::net::SocketAddr;

use common::config::SERVER_PORT;

pub fn server_addr() -> SocketAddr {
    std::fs::read_to_string("server_addr.txt")
        .map(|s| s.trim().parse().unwrap())
        .unwrap_or_else(|_| {
            local_ip_address::local_ip()
                .map(|ip| format!("{}:{}", ip, SERVER_PORT))
                .unwrap()
        })
        .parse()
        .unwrap()
}
