use std::{net::UdpSocket, time::Duration};

use common::{
    debug_draw_config::DebugDrawConfig, draw::Sprites,
    draw_server_controlled_game_state::draw_server_controlled_game_state,
    game_state::ServerControlledGameState, hit_numbers::HitNumbers,
    message_acknowledgement::AckUdpSocket, network::ServerMessage,
};
use macroquad::window::next_frame;
use serde::{Deserialize, Serialize};

pub mod condition;
pub mod test_basic_movement_and_attack;
pub mod test_environment;
pub mod test_protector_can_attack_ranger;

pub const TEST_CLIENT_ADDR: &str = "127.0.0.1:12346";

#[derive(Debug, Serialize, Deserialize)]
pub struct TestMonitorPing {}

pub struct TestMonitorState {
    ack_udp_socket: AckUdpSocket<TestMonitorPing, ServerMessage>,
    server_controlled_game_state: ServerControlledGameState,
    sprites: Sprites,
    debug_draw_config: DebugDrawConfig,
    hit_numbers: HitNumbers,
}

impl TestMonitorState {
    async fn new() -> Self {
        let udp_socket = UdpSocket::bind(TEST_CLIENT_ADDR).unwrap();
        udp_socket.set_nonblocking(true).unwrap();
        let ack_udp_socket =
            AckUdpSocket::<TestMonitorPing, ServerMessage>::new(udp_socket, Duration::from_secs(1));
        let server_controlled_game_state = ServerControlledGameState::default();
        Self {
            ack_udp_socket,
            server_controlled_game_state,
            sprites: Sprites::load().await,
            debug_draw_config: DebugDrawConfig { draw_paths: true },
            hit_numbers: HitNumbers::new(),
        }
    }
}

#[macroquad::main("Test Monitor")]
async fn main() {
    let mut state = TestMonitorState::new().await;
    loop {
        while let Some((server_message, server_addr)) = state.ack_udp_socket.receive() {
            state
                .server_controlled_game_state
                .update_with_server_message(server_message);
            state
                .ack_udp_socket
                .send_to(TestMonitorPing {}, &server_addr, false);
        }

        state.hit_numbers.step(
            &state
                .server_controlled_game_state
                .dynamic_game_state
                .entities,
            0.016,
        );
        draw_server_controlled_game_state(
            &state.server_controlled_game_state,
            &state.sprites,
            &state.debug_draw_config,
        );
        state.hit_numbers.draw(None);

        next_frame().await;
    }
}
