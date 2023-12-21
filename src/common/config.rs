use std::net::SocketAddr;

pub const PROJECTILE_RADIUS: f32 = 12.0;
pub const TARGET_SERVER_FPS: f32 = 60.0;

pub fn server_addr() -> SocketAddr {
    std::fs::read_to_string("server_addr().txt")
        .map(|s| s.trim().parse().unwrap())
        .unwrap_or_else(|_| {
            local_ip_address::local_ip()
                .map(|ip| format!("{}:7878", ip))
                .unwrap()
        })
        .parse()
        .unwrap()
}
