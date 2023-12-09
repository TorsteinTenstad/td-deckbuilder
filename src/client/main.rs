use common::card::Card;
use common::*;
use local_ip_address::local_ip;
use macroquad::{
    input::{is_mouse_button_down, is_mouse_button_released, MouseButton},
    prelude::Vec2,
    window::next_frame,
    window::request_new_screen_size,
};
mod draw;
use draw::*;
mod input;
use input::*;
mod network;
use network::*;
mod player;
use player::*;
use std::net::{SocketAddr, UdpSocket};
use std::time::SystemTime;

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
}

impl ClientGameState {
    pub fn new() -> Self {
        let local_ip = local_ip().unwrap();
        let udp_socket = std::iter::successors(Some(6968), |port| Some(port + 1))
            .find_map(|port| {
                let socket_addr = SocketAddr::new(local_ip, port);
                UdpSocket::bind(socket_addr).ok()
            })
            .unwrap();
        udp_socket.set_nonblocking(true).unwrap();
        let player_id = hash_client_addr(&udp_socket.local_addr().unwrap());

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
        }
    }
}

fn tower_at_tile(state: &ClientGameState, pos: Vec2) -> Option<&Entity> {
    state.dynamic_game_state.entities.values().find(|entity| {
        entity.tag == EntityTag::Tower
            && entity.pos.x as i32 == pos.x as i32
            && entity.pos.y as i32 == pos.y as i32
    })
}

#[macroquad::main("Client")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);

    let textures = load_textures().await;
    let mut state = ClientGameState::new();

    loop {
        let old_time = state.time;
        state.time = SystemTime::now();
        state.dt = state.time.duration_since(old_time).unwrap().as_secs_f32();

        udp_update_game_state(&mut state);
        main_input(&mut state);
        udp_send_commands(&mut state);
        main_draw(&state);

        let input = &state.input;

        let highlighted_card_opt_clone = state.highlighted_card_opt.clone();
        state.highlighted_card_opt = None;

        for (i, card) in state.hand.hand.iter().enumerate() {
            let is_selected = highlighted_card_opt_clone == Some(i);

            // let hovering = card_is_hovering(x, y, w, h, rotation, offset);
            // if !(is_selected && !is_mouse_button_down(MouseButton::Left)) {
            draw_in_hand_card(
                card,
                i,
                state.hand.hand.len(),
                if is_selected { 0.5 } else { 1.0 },
                state.relative_splay_radius,
                state.card_delta_angle,
                &textures,
            );
            // }
            // if hovering {
            //     state.highlighted_card_opt = Some(i);
            // }
        }

        let mouse_pos = mouse_position_vec();
        let mouse_world_pos = mouse_world_position();

        if let Some(highlighted_card) = highlighted_card_opt_clone {
            let card = state.hand.hand.get(highlighted_card).unwrap();

            if is_mouse_button_released(MouseButton::Left) {
                if input.mouse_in_world
                    && match card {
                        Card::BasicTower => !input.mouse_over_occupied_tile,
                        Card::BasicRanger | Card::BasicDrone | Card::BasicUnit => true,
                    }
                {
                    if let Some(card) = state
                        .hand
                        .try_move_card_from_hand_to_played(highlighted_card)
                    {
                        state.commands.push(ClientCommand::PlayCard(
                            mouse_world_pos.x,
                            mouse_world_pos.y,
                            card.clone(),
                        ));
                    }
                }
                state.preview_tower_pos = None;
            } else {
                if is_mouse_button_down(MouseButton::Left) {
                    state.highlighted_card_opt = Some(highlighted_card);
                    if input.mouse_in_world {
                        match card {
                            Card::BasicTower => {
                                state.preview_tower_pos = Some((
                                    mouse_world_pos.x as i32 as f32 + 0.5,
                                    mouse_world_pos.y as i32 as f32 + 0.5,
                                ));
                            }
                            Card::BasicRanger | Card::BasicDrone | Card::BasicUnit => {
                                draw_out_of_hand_card(card, mouse_pos.x, mouse_pos.y, &textures);
                            }
                        }
                    } else {
                        draw_out_of_hand_card(card, mouse_pos.x, mouse_pos.y, &textures);
                    }
                } else {
                    draw_highlighted_card(
                        card,
                        highlighted_card,
                        state.relative_splay_radius,
                        state.card_delta_angle,
                        &textures,
                        state.hand.hand.len(),
                    );
                    // if hovering {
                    //     state.highlighted_card_opt = Some(highlighted_card);
                    // }
                }
            }
        }

        next_frame().await;
    }
}
