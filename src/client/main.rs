use common::play_target::UnitSpawnpointTarget;
use common::*;
use macroquad::color::{Color, RED};
use macroquad::math::Vec2;
use macroquad::shapes::{draw_rectangle, draw_rectangle_ex, DrawRectangleParams};
use macroquad::{texture::Texture2D, window::next_frame, window::request_new_screen_size};
pub mod config;
mod draw;
use draw::*;
mod input;
use input::*;
mod network;
use network::*;
mod player;
use player::*;
use std::time::SystemTime;
use std::{collections::HashMap, net::UdpSocket};

pub struct ClientGameState {
    static_game_state: StaticGameState,
    dynamic_game_state: DynamicGameState,
    time: SystemTime,
    selected_entity_id: Option<u64>,
    hand: Hand,
    relative_splay_radius: f32,
    card_delta_angle: f32,
    highlighted_card_opt: Option<usize>,
    preview_tower_pos: Option<(f32, f32)>,
    frames_since_last_received: i32,
    commands: Vec<ClientCommand>,
    udp_socket: UdpSocket,
    player_id: u64,
    input: GameInput,
    dt: f32,
    textures: HashMap<String, Texture2D>,
    unit_spawnpoint_targets: Vec<UnitSpawnpointTarget>,
}

impl ClientGameState {
    pub async fn new() -> Self {
        let (udp_socket, player_id) = udp_init_socket();

        Self {
            static_game_state: StaticGameState::new(),
            dynamic_game_state: DynamicGameState::new(),
            time: SystemTime::now(),
            card_delta_angle: 0.1,
            relative_splay_radius: 4.5,
            commands: Vec::new(),
            frames_since_last_received: 0,
            hand: Hand::new(),
            highlighted_card_opt: None,
            preview_tower_pos: None,
            selected_entity_id: None,
            udp_socket,
            player_id,
            input: GameInput::default(),
            dt: 0.167,
            textures: load_textures().await,
            unit_spawnpoint_targets: Vec::new(),
        }
    }
}

#[macroquad::main("Client")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);

    let mut state = ClientGameState::new().await;

    loop {
        let old_time = state.time;
        state.time = SystemTime::now();
        state.dt = state.time.duration_since(old_time).unwrap().as_secs_f32();

        udp_update_game_state(&mut state);
        main_input(&mut state);
        udp_send_commands(&mut state);
        main_draw(&state);
        for physical_card in state.hand.hand.iter_mut() {
            draw_card(
                &physical_card.card,
                &physical_card.transform,
                1.0,
                &state.textures,
            )
        }
        for target in state.unit_spawnpoint_targets.iter() {
            let transform =
                &unit_spawnpoint_gui_indicator_transform(target, &state.static_game_state);
            let hovering = curser_is_inside(transform);
            draw_rect_transform(
                transform,
                Color {
                    a: if hovering { 0.8 } else { 0.5 },
                    ..RED
                },
            );
        }
        player_step(&mut state);

        next_frame().await;
    }
}
