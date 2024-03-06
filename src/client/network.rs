use crate::config::default_server_addr;
use common::{
    ids::PlayerId,
    message_acknowledgement::AckUdpSocket,
    network::{hash_client_addr, ClientMessage, ServerMessage},
};
use local_ip_address::local_ip;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

pub struct ClientNetworkState {
    pub server_addr: SocketAddr,
    ack_udp_socket: AckUdpSocket<ClientMessage, ServerMessage>,
    last_server_com: Option<SystemTime>,
}

impl ClientNetworkState {
    pub fn new() -> Self {
        let local_ip = local_ip().unwrap();
        let udp_socket = std::iter::successors(Some(6968), |port| Some(port + 1))
            .find_map(|port| {
                let socket_addr = SocketAddr::new(local_ip, port);
                UdpSocket::bind(socket_addr).ok()
            })
            .unwrap();
        udp_socket.set_nonblocking(true).unwrap();

        Self {
            server_addr: default_server_addr(),
            ack_udp_socket: AckUdpSocket::new(udp_socket, std::time::Duration::from_secs(1)),
            last_server_com: None,
        }
    }

    pub fn ensure_joined(&mut self, join_message: ClientMessage) {
        if self
            .last_server_com
            .is_some_and(|time| time.elapsed().unwrap().as_secs() < 1)
        {
            return;
        }
        self.last_server_com = Some(SystemTime::now());
        self.ack_udp_socket
            .send_to(join_message, &self.server_addr, false); // No ack needed: If the server doesn't respond, we'll just try again
    }

    pub fn push_command(&mut self, client_message: ClientMessage) {
        self.ack_udp_socket
            .queue(client_message, &self.server_addr, true);
    }

    pub fn receive(&mut self) -> Option<ServerMessage> {
        let response = self.ack_udp_socket.receive().map(|(message, _)| message);
        if response.is_some() {
            self.last_server_com = Some(SystemTime::now());
        }
        response
    }

    pub fn send_queued(&mut self) {
        self.ack_udp_socket.send_queued();
    }

    pub fn get_player_id(&self) -> PlayerId {
        hash_client_addr(&self.ack_udp_socket.local_addr().unwrap())
    }
}
