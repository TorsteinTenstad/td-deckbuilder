use crate::ClientGameState;
use common::{
    ids::PlayerId,
    network::{hash_client_addr, ClientMessage, ServerMessage},
};
use local_ip_address::local_ip;
use macroquad::input::is_key_down;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

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
        let received_message = state.udp_socket.recv_from(&mut buf);
        match received_message {
            Ok((number_of_bytes, _src)) => {
                dbg!(number_of_bytes);
                state.frames_since_last_received = 0;
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
                    Ok(server_message) => state.update_with_server_message(server_message),
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

    state.frames_since_last_received += 1;
    if state.frames_since_last_received > 60 {
        state
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
                state.server_addr,
            )
            .unwrap();
    }
    if !state.static_game_state_received {
        state
            .udp_socket
            .send_to(
                &rmp_serde::to_vec(&ClientMessage::RequestStaticGameState).unwrap(),
                state.server_addr,
            )
            .unwrap();
    }
}

pub fn udp_send_commands(state: &mut ClientGameState) {
    for command in &state.commands {
        state
            .udp_socket
            .send_to(
                &rmp_serde::to_vec(&command).unwrap().as_slice(),
                state.server_addr,
            )
            .unwrap();
    }
    state.commands.clear();
}
