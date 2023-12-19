use common::card::CardInstance;
use common::play_target::{PlayFn, UnitSpawnpointTarget};
use common::*;
use macroquad::color::{Color, BLACK, RED, WHITE};
use macroquad::math::Vec2;
use macroquad::shapes::draw_circle_lines;
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
use macroquad::window::{clear_background, screen_height, screen_width};
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

#[derive(Default)]
pub struct PhysicalHand {
    card_idx_being_held: Option<usize>,
    cards: Vec<PhysicalCard>,
}

pub fn update_from_hand(state: &mut ClientGameState) {
    // TODO: Find a way to remove clone?
    let server_hand = state.get_player().hand.cards.clone();
    for card_instance in server_hand.iter() {
        let physical_card = state
            .physical_hand
            .cards
            .iter_mut()
            .find(|c| c.card_instance.id == card_instance.id);
        if let Some(physical_card) = physical_card {
            physical_card.card_instance = card_instance.clone();
        } else {
            state
                .physical_hand
                .cards
                .push(PhysicalCard::new(card_instance.clone()));
        }
    }
    state.physical_hand.cards.retain(|physical_card| {
        server_hand
            .iter()
            .any(|card_instance| card_instance.id == physical_card.card_instance.id)
    });
}
pub fn hand_try_play(state: &ClientGameState) -> Option<CardInstance> {
    let Some(card_idx_being_held) = state.physical_hand.card_idx_being_held else {
        return None;
    };
    let card_instance = state
        .physical_hand
        .cards
        .get(card_idx_being_held)
        .unwrap()
        .card_instance
        .clone();
    if state.get_player().hand.energy < card_instance.card.energy_cost() {
        return None;
    }
    Some(card_instance)
}

pub struct ClientGameState {
    static_game_state: StaticGameState,
    dynamic_game_state: DynamicGameState,
    time: SystemTime,
    selected_entity_id: Option<u64>,
    relative_splay_radius: f32,
    card_delta_angle: f32,
    preview_tower_pos: Option<(f32, f32)>,
    frames_since_last_received: i32,
    commands: Vec<ClientCommand>,
    udp_socket: UdpSocket,
    player_id: u64,
    input: GameInput,
    dt: f32,
    textures: HashMap<String, Texture2D>,
    unit_spawnpoint_targets: Vec<UnitSpawnpointTarget>,
    physical_hand: PhysicalHand,
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
            preview_tower_pos: None,
            selected_entity_id: None,
            udp_socket,
            player_id,
            input: GameInput::default(),
            dt: 0.167,
            textures: load_textures().await,
            unit_spawnpoint_targets: Vec::new(),
            physical_hand: PhysicalHand::default(),
        }
    }
    pub fn get_player(&self) -> &ServerPlayer {
        self.dynamic_game_state
            .players
            .get(&self.player_id)
            .unwrap()
    }
    pub fn get_mut_player(&mut self) -> &mut ServerPlayer {
        self.dynamic_game_state
            .players
            .get_mut(&self.player_id)
            .unwrap()
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

        // board
        clear_background(BLACK);

        draw_texture_ex(
            state.textures.get("concept").unwrap(),
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: screen_width(),
                    y: screen_height(),
                }),
                ..Default::default()
            },
        );
        for physical_card in state.physical_hand.cards.iter_mut() {
            draw_card(
                &physical_card.card_instance.card,
                &physical_card.transform,
                1.0,
                &state.textures,
            )
        }
        main_draw(&state);
        if state
            .physical_hand
            .card_idx_being_held
            .filter(|idx| {
                matches!(
                    state.physical_hand.cards[*idx]
                        .card_instance
                        .card
                        .get_card_data()
                        .play_fn,
                    PlayFn::BuildingSpot(_)
                )
            })
            .is_some()
        {
            for (_id, loc) in state.dynamic_game_state.building_locations.iter() {
                let x = to_screen_x(loc.position.0);
                let y = to_screen_y(loc.position.1);
                let r = 20.0;
                let hovering = (mouse_position_vec() - Vec2 { x, y }).length() < r;
                draw_circle_lines(
                    x,
                    y,
                    r,
                    3.0,
                    Color {
                        a: if hovering { 0.8 } else { 0.5 },
                        ..RED
                    },
                );
            }
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
