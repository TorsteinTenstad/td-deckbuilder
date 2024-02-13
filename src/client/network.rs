use crate::{config::default_server_addr, ClientGameState};
use common::{
    game_state::ServerControledGameState,
    ids::PlayerId,
    network::{hash_client_addr, ClientMessage, ServerMessage},
};
use local_ip_address::local_ip;
use macroquad::input::is_key_down;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

pub struct ClientNetworkState {
    pub server_addr: SocketAddr,
    pub udp_socket: UdpSocket,
    pub commands: Vec<ClientMessage>,
    pub frames_since_last_received: i32,
    pub static_game_state_received: bool,
}

impl ClientNetworkState {
    pub fn new(udp_socket: UdpSocket) -> Self {
        Self {
            server_addr: default_server_addr(),
            commands: Vec::new(),
            frames_since_last_received: 0,
            static_game_state_received: false,
            udp_socket,
        }
    }
}

pub fn udp_init_socket() -> (UdpSocket, PlayerId) {
    let local_ip = local_ip().unwrap();
    let udp_socket = std::iter::successors(Some(6968), |port| Some(port + 1))
        .find_map(|port| {
            let socket_addr = SocketAddr::new(local_ip, port);
            UdpSocket::bind(socket_addr).ok()
        })
        .unwrap();
    udp_socket.set_nonblocking(true).unwrap();
    let player_id = hash_client_addr(&udp_socket.local_addr().unwrap());

    (udp_socket, player_id)
}

pub fn udp_update_game_state(state: &mut ClientGameState) {
    loop {
        let mut buf = [0; 20000];
        let received_message = state.client_network_state.udp_socket.recv_from(&mut buf);
        match received_message {
            Ok((number_of_bytes, _src)) => {
                dbg!(number_of_bytes);
                state.client_network_state.frames_since_last_received = 0;
                let buf = &mut buf[..number_of_bytes];
                let log = |prefix: &str| {
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    std::fs::write(format!("{}client_recv_{}.txt", prefix, timestamp), &buf)
                        .unwrap();
                };
                if is_key_down(macroquad::prelude::KeyCode::F11) {
                    log("");
                }
                let deserialization_result = rmp_serde::from_slice::<ServerMessage>(buf);
                match deserialization_result {
                    Err(e) => {
                        log("error_");
                        dbg!(e);
                        panic!()
                    }
                    Ok(server_message) => {
                        state
                            .update_server_controled_game_state_with_server_message(server_message);
                    }
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::ConnectionReset => {
                    break;
                }
                _ => {
                    dbg!(e);
                    panic!()
                }
            },
        }
    }

    state.client_network_state.frames_since_last_received += 1;
    if state.client_network_state.frames_since_last_received > 60 {
        state
            .client_network_state
            .udp_socket
            .send_to(
                &rmp_serde::to_vec(&ClientMessage::JoinGame(
                    state
                        .deck_builder
                        .deck
                        .iter()
                        .map(|physical_card| physical_card.card.clone())
                        .collect(),
                ))
                .unwrap(),
                state.client_network_state.server_addr,
            )
            .unwrap();
    }
    if !state.client_network_state.static_game_state_received {
        state
            .client_network_state
            .udp_socket
            .send_to(
                &rmp_serde::to_vec(&ClientMessage::RequestStaticGameState).unwrap(),
                state.client_network_state.server_addr,
            )
            .unwrap();
    }
}

pub fn udp_send_commands(client_network_state: &mut ClientNetworkState) {
    for command in &client_network_state.commands {
        client_network_state
            .udp_socket
            .send_to(
                &rmp_serde::to_vec(&command).unwrap().as_slice(),
                client_network_state.server_addr,
            )
            .unwrap();
    }
    client_network_state.commands.clear();
}
