use crate::{physical_hand::hand_sync, ClientGameState, PhysicalHand};
use common::{
    config::server_addr,
    game_state::ServerGameState,
    ids::PlayerId,
    network::{hash_client_addr, ClientCommand},
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
                state.frames_since_last_received = 0;
                let buf = &mut buf[..number_of_bytes];
                let log = |prefix: &str| {
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    std::fs::write(format!("{}client_recv_{}.json", prefix, timestamp), &buf)
                        .unwrap();
                };
                if is_key_down(macroquad::prelude::KeyCode::F11) {
                    log("");
                }
                let deserialization_result = serde_json::from_slice::<ServerGameState>(buf); //TODO: handle error
                if let Err(e) = deserialization_result {
                    log("error_");
                    dbg!(e);
                    panic!()
                }
                let received_game_state = deserialization_result.unwrap();
                if received_game_state.dynamic_state.server_tick
                    > state.dynamic_game_state.server_tick
                    || received_game_state.static_state.game_id != state.static_game_state.game_id
                {
                    if received_game_state.static_state.game_id != state.static_game_state.game_id {
                        state.physical_hand = PhysicalHand::default();
                    }
                    state.dynamic_game_state = received_game_state.dynamic_state;
                    state.static_game_state = received_game_state.static_state;

                    hand_sync(state);
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

    if state.frames_since_last_received > 60 {
        state
            .udp_socket
            .send_to(
                &serde_json::to_string(&ClientCommand::JoinGame)
                    .unwrap()
                    .as_bytes(),
                server_addr(),
            )
            .unwrap();
    }
    state.frames_since_last_received += 1;
}

pub fn udp_send_commands(state: &mut ClientGameState) {
    for command in &state.commands {
        state
            .udp_socket
            .send_to(
                &serde_json::to_string(&command).unwrap().as_bytes(),
                server_addr(),
            )
            .unwrap();
    }
    state.commands.clear();
}
