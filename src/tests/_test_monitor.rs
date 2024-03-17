use std::{net::UdpSocket, time::Duration};

use common::{
    debug_draw_config::DebugDrawConfig, draw::Sprites,
    draw_server_controlled_game_state::draw_server_controlled_game_state,
    game_state::ServerControlledGameState, message_acknowledgement::AckUdpSocket,
    network::ServerMessage,
};
use macroquad::window::next_frame;
use serde::{Deserialize, Serialize};

pub mod test_basic_movement_and_attack;
pub mod test_environment;

pub const TEST_SERVER_ADDR: &str = "127.0.0.1:12345";
pub const TEST_CLIENT_ADDR: &str = "127.0.0.1:12346";

#[derive(Debug, Serialize, Deserialize)]
pub struct TestMonitorPing {}

#[macroquad::main("Test Monitor")]
async fn main() {
    let udp_socket = UdpSocket::bind(TEST_CLIENT_ADDR).unwrap();
    udp_socket.set_nonblocking(true).unwrap();
    let mut ack_udp_socket =
        AckUdpSocket::<TestMonitorPing, ServerMessage>::new(udp_socket, Duration::from_secs(1));
    let mut server_controlled_game_state = ServerControlledGameState::default();
    let sprites = Sprites::load().await;
    let test_server_addr = TEST_SERVER_ADDR.parse().unwrap();

    loop {
        ack_udp_socket.send_to(TestMonitorPing {}, &test_server_addr, false);

        while let Some((server_message, _)) = ack_udp_socket.receive() {
            server_controlled_game_state.update_with_server_message(server_message);
        }

        draw_server_controlled_game_state(
            &server_controlled_game_state,
            &sprites,
            &DebugDrawConfig::default(),
        );

        next_frame().await;
    }
}
