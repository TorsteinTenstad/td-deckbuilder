use common::card::Card;
use common::*;
use local_ip_address::local_ip;
use macroquad::prelude::Vec2;
use macroquad::shapes::{draw_poly, draw_triangle};
use macroquad::texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D};
use macroquad::{
    color::{Color, BLACK, BLUE, GRAY, LIGHTGRAY, RED, WHITE, YELLOW},
    input::{
        is_key_down, is_mouse_button_down, is_mouse_button_released, mouse_position, MouseButton,
    },
    shapes::{
        draw_circle, draw_circle_lines, draw_hexagon, draw_rectangle_ex, DrawRectangleParams,
    },
    window::request_new_screen_size,
    window::{clear_background, next_frame, screen_height, screen_width},
};
mod draw;
use draw::*;
use rand::Rng;
use std::collections::HashMap;
use std::default;
use std::net::{SocketAddr, UdpSocket};
use std::time::SystemTime;

const GOLDEN_RATIO: f32 = 1.61803398875;

const GRASS_COLOR: Color = Color {
    r: 0.686,
    g: 0.784,
    b: 0.490,
    a: 1.0,
};

const PATH_COLOR: Color = Color {
    r: 0.843,
    g: 0.803,
    b: 0.627,
    a: 1.0,
};

fn shuffle_vec<T>(vec: &mut Vec<T>) {
    let mut rng = rand::thread_rng();
    for i in 0..vec.len() {
        let j = rng.gen_range(0..vec.len());
        vec.swap(i, j);
    }
}

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

#[derive(Default)]
pub struct GameInput {
    mouse_position: Vec2,
    mouse_world_x: f32,
    mouse_world_y: f32,
    mouse_in_world: bool,
    mouse_over_occupied_tile: bool,
}

pub struct Hand {
    card_draw_counter: i32,
    energy_counter: i32,
    energy: i32,
    hand: Vec<Card>,
    deck: Vec<Card>,
    played: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        let mut deck = Vec::new();
        for (quantity, card) in vec![
            (3, Card::BasicTower),
            (5, Card::BasicUnit),
            (3, Card::BasicDrone),
            (3, Card::BasicRanger),
        ] {
            for _ in 0..quantity {
                deck.push(card.clone());
            }
        }
        shuffle_vec(&mut deck);
        Self {
            card_draw_counter: 0,
            energy_counter: 0,
            energy: 0,
            hand: Vec::new(),
            deck,
            played: Vec::new(),
        }
    }

    pub fn sync_with_server_counters(
        &mut self,
        server_card_draw_counter: i32,
        server_energy_counter: i32,
    ) {
        while self.card_draw_counter < server_card_draw_counter {
            self.draw();
            self.card_draw_counter += 1;
        }
        while self.energy_counter < server_energy_counter {
            self.energy = (self.energy + 1).min(10);
            self.energy_counter += 1;
        }
    }

    pub fn draw(&mut self) -> Option<Card> {
        if self.hand.len() >= 10 {
            return None;
        }
        if self.deck.is_empty() {
            self.deck = self.played.clone();
            self.played.clear();
            shuffle_vec(&mut self.deck);
        }
        let card = self.deck.pop().unwrap();
        self.hand.push(card.clone());
        Some(card)
    }

    pub fn try_move_card_from_hand_to_played(&mut self, index: usize) -> Option<Card> {
        if self.energy < self.hand.get(index).unwrap().energy_cost() {
            return None;
        }
        let card = self.hand.remove(index);
        self.energy -= card.energy_cost();
        self.played.push(card.clone());
        Some(card)
    }
}

fn receive_game_state(state: &mut ClientGameState) {
    loop {
        let mut buf = [0; 20000];
        let received_message = state.udp_socket.recv_from(&mut buf);
        match received_message {
            Ok((amt, _src)) => {
                state.frames_since_last_received = 0;
                let buf = &mut buf[..amt];
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
                        state.hand = Hand::new();
                    }
                    state.dynamic_game_state = received_game_state.dynamic_state;
                    state.static_game_state = received_game_state.static_state;
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
    if let Some(player) = state.dynamic_game_state.players.get(&state.player_id) {
        state.hand.sync_with_server_counters(
            player.card_draw_counter as i32,
            player.energy_counter as i32,
        );
    }
    if state.frames_since_last_received > 60 {
        state
            .udp_socket
            .send_to(
                &serde_json::to_string(&ClientCommand::JoinGame)
                    .unwrap()
                    .as_bytes(),
                SERVER_ADDR,
            )
            .unwrap();
    }
}

// TODO overloading?
fn cell_w() -> f32 {
    screen_width() / 16.0 // state.static_game_state.grid_w as f32; TODO
}
fn cell_h() -> f32 {
    (7.0 / 9.0) * screen_height() / 7.0 // state.static_game_state.grid_h as f32; TODO
}
fn u32_to_screen_x(x: u32) -> f32 {
    (x as f32) * cell_w()
}
fn u32_to_screen_y(y: u32) -> f32 {
    (y as f32) * cell_h()
}
fn f32_to_screen_x(x: f32) -> f32 {
    (x as f32) * cell_w()
}
fn f32_to_screen_y(y: f32) -> f32 {
    (y as f32) * cell_h()
}
fn to_screen_size(x: f32) -> f32 {
    x * cell_w()
}

fn tower_at_tile(state: &ClientGameState, x: i32, y: i32) -> Option<&Entity> {
    state.dynamic_game_state.entities.values().find(|entity| {
        entity.tag == EntityTag::Tower && entity.pos.x as i32 == x && entity.pos.y as i32 == y
    })
}

fn get_input(state: &mut ClientGameState) {
    state.input.mouse_position = Vec2::from_array(mouse_position().into());

    state.input.mouse_world_x = state.input.mouse_position.x / cell_w();
    state.input.mouse_world_y = state.input.mouse_position.y / cell_h();
    state.input.mouse_in_world = state.input.mouse_world_x >= 0.0
        && state.input.mouse_world_x >= 0.0
        && (state.input.mouse_world_x as u32) < state.static_game_state.grid_w
        && (state.input.mouse_world_y as u32) < state.static_game_state.grid_h;
    state.input.mouse_over_occupied_tile = tower_at_tile(
        &state,
        state.input.mouse_world_x as i32,
        state.input.mouse_world_y as i32,
    )
    .is_some();

    if is_mouse_button_released(MouseButton::Left) {
        state.selected_entity_id =
            state
                .dynamic_game_state
                .entities
                .iter()
                .find_map(|(id, entity)| {
                    ((entity.pos
                        - Vec2 {
                            x: state.input.mouse_world_x,
                            y: state.input.mouse_world_y,
                        })
                    .length()
                        < entity.radius)
                        .then(|| *id)
                });
    }

    //Card drawing parameter adjustment
    {
        if is_key_down(macroquad::prelude::KeyCode::L) {
            state.card_delta_angle += 0.05 * state.dt;
            dbg!(state.card_delta_angle);
        }
        if is_key_down(macroquad::prelude::KeyCode::J) {
            state.card_delta_angle -= 0.05 * state.dt;
            dbg!(state.card_delta_angle);
        }
        if is_key_down(macroquad::prelude::KeyCode::I) {
            state.relative_splay_radius += 0.5 * state.dt;
            dbg!(state.relative_splay_radius);
        }
        if is_key_down(macroquad::prelude::KeyCode::K) {
            state.relative_splay_radius -= 0.5 * state.dt;
            dbg!(state.relative_splay_radius);
        }
    }
}

fn udp_send_commands(state: &mut ClientGameState) {
    for command in &state.commands {
        state
            .udp_socket
            .send_to(
                &serde_json::to_string(&command).unwrap().as_bytes(),
                SERVER_ADDR,
            )
            .unwrap();
    }
    state.commands.clear();
}

const CARD_BORDER: f32 = 5.0;
const CARD_VISIBLE_HEIGHT: f32 = 0.8;

fn draw_card(
    card: &Card,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    offset: Vec2,
    alpha: f32,
    only_compute_hover: bool,
    textures: &HashMap<String, Texture2D>,
    mouse_position: Vec2,
) -> bool {
    draw_circle(x, y, 3.0, YELLOW);
    let local_mouse_pos: Vec2 = Vec2::from_angle(-rotation).rotate(mouse_position - Vec2 { x, y })
        + offset * Vec2 { x: w, y: h };
    let hovering =
        local_mouse_pos.cmpgt(Vec2::ZERO).all() && local_mouse_pos.cmplt(Vec2::new(w, h)).all();
    if only_compute_hover {
        return hovering;
    }
    draw_rectangle_ex(
        x,
        y,
        w,
        h,
        DrawRectangleParams {
            color: Color { a: alpha, ..GRAY },
            rotation,
            offset,
            ..Default::default()
        },
    );
    let inner_offset = offset
        + Vec2 {
            x: 0.0,
            y: (2.0 * offset.y - 1.0) * CARD_BORDER / h,
        };
    draw_rectangle_ex(
        x,
        y,
        w - 2.0 * CARD_BORDER,
        h - 2.0 * CARD_BORDER,
        DrawRectangleParams {
            color: Color {
                a: alpha,
                ..LIGHTGRAY
            },
            rotation,
            offset: inner_offset,
            ..Default::default()
        },
    );

    let get_on_card_pos = |rel_x, rel_y| -> Vec2 {
        Vec2 { x, y }
            + Vec2::from_angle(rotation)
                .rotate(Vec2 { x: w, y: h } * (Vec2 { x: rel_x, y: rel_y } - offset))
    };

    let card_name_pos = get_on_card_pos(0.9, 0.1);
    draw_text_with_origin(
        card.name(),
        card_name_pos.x,
        card_name_pos.y,
        20.0,
        rotation,
        BLACK,
        TextOriginX::Right,
        TextOriginY::Top,
    );

    let width_relative_margin = 0.1;
    let energy_indicator_pos =
        get_on_card_pos(width_relative_margin, width_relative_margin * w / h);

    let mut icons: Vec<(&str, f32)> = Vec::new();
    let entity = card.to_entity(
        0,
        &ServerPlayer::new(Direction::Positive, Vec2::ZERO, BLACK),
        0.0,
        0.0,
    );

    icons.push(("shield", entity.health));
    if let Some(RangedAttack {
        range,
        damage,
        fire_interval,
        ..
    }) = entity.ranged_attack
    {
        icons.push(("bow", damage));
        icons.push(("range", range));
        icons.push(("hourglass_bow", fire_interval));
    }
    if let Some(MeleeAttack {
        damage,
        attack_interval,
        ..
    }) = entity.melee_attack
    {
        icons.push(("sword", damage));
        icons.push(("hourglass_sword", attack_interval));
    };

    for (i, (texture_id, value)) in icons.iter().filter(|(_, value)| *value > 0.001).enumerate() {
        let width_relative_icon_size = 0.2;
        let on_card_icon_pos = get_on_card_pos(
            width_relative_margin,
            2.0 * width_relative_margin + i as f32 * (width_relative_icon_size),
        );
        let on_card_value_pos = get_on_card_pos(
            2.0 * width_relative_margin + width_relative_icon_size,
            2.0 * width_relative_margin + (i as f32 + 0.25) * (width_relative_icon_size),
        );
        let icon_size = Vec2::splat(w * width_relative_icon_size);
        let texture = textures
            .get(*texture_id)
            .expect(format!("Texture \"{}\" not found", texture_id).as_str());
        draw_texture_ex(
            texture,
            on_card_icon_pos.x,
            on_card_icon_pos.y,
            WHITE,
            DrawTextureParams {
                rotation,
                dest_size: Some(icon_size),
                pivot: Some(on_card_icon_pos + icon_size / 2.0),
                ..Default::default()
            },
        );
        draw_text_with_origin(
            format!("{}", value).as_str(),
            on_card_value_pos.x,
            on_card_value_pos.y,
            26.0,
            rotation,
            BLACK,
            TextOriginX::Left,
            TextOriginY::AbsoluteCenter,
        );
    }

    draw_circle(
        energy_indicator_pos.x,
        energy_indicator_pos.y,
        w / 8.0,
        BLUE,
    );
    draw_text_with_origin(
        format!("{}", card.energy_cost()).as_str(),
        energy_indicator_pos.x,
        energy_indicator_pos.y,
        24.0,
        rotation,
        WHITE,
        TextOriginX::Center,
        TextOriginY::AbsoluteCenter,
    );
    hovering
}

fn draw_in_hand_card(
    card: &Card,
    i: usize,
    n: usize,
    alpha: f32,
    only_compute_hover: bool,
    relative_splay_radius: f32,
    card_delta_angle: f32,
    textures: &HashMap<String, Texture2D>,
    mouse_position: Vec2,
) -> bool {
    let card_w = screen_width() / 12.0;
    let card_h = card_w * GOLDEN_RATIO;
    let x = screen_width() / 2.0;
    let y = screen_height() + (relative_splay_radius * card_h) - (CARD_VISIBLE_HEIGHT * card_h);
    let rotation = (i as f32 - ((n - 1) as f32 / 2.0)) * card_delta_angle;
    let offset = Vec2 {
        x: 0.5,
        y: relative_splay_radius,
    };
    draw_card(
        card,
        x,
        y,
        card_w,
        card_h,
        rotation,
        offset,
        alpha,
        only_compute_hover,
        textures,
        mouse_position,
    )
}

fn draw_out_of_hand_card(
    card: &Card,
    x: f32,
    y: f32,
    textures: &HashMap<String, Texture2D>,
    mouse_position: Vec2,
) -> bool {
    let card_w = screen_width() / 10.0;
    let card_h = card_w * GOLDEN_RATIO;
    draw_card(
        card,
        x,
        y,
        card_w,
        card_h,
        0.0,
        0.5 * Vec2::ONE,
        1.0,
        false,
        textures,
        mouse_position,
    )
}

fn draw_highlighted_card(
    card: &Card,
    i: usize,
    relative_splay_radius: f32,
    card_delta_angle: f32,
    textures: &HashMap<String, Texture2D>,
    mouse_position: Vec2,
    hand_length: usize,
) -> bool {
    let card_w = screen_width() / 10.0;
    let card_h = card_w * GOLDEN_RATIO;
    let x = screen_width() / 2.0
        + ((relative_splay_radius * card_h) - (CARD_VISIBLE_HEIGHT * card_h))
            * f32::sin((i as f32 - ((hand_length - 1) as f32 / 2.0)) * card_delta_angle);
    let y = screen_height();
    draw_card(
        card,
        x,
        y,
        card_w,
        card_h,
        0.0,
        Vec2 { x: 0.5, y: 1.0 },
        1.0,
        false,
        textures,
        mouse_position,
    )
}

async fn load_textures() -> HashMap<String, Texture2D> {
    let mut textures: HashMap<String, Texture2D> = HashMap::new();
    for texture_id in vec![
        "hourglass",
        "hourglass_bow",
        "hourglass_sword",
        "range",
        "shield",
        "sword",
        "bow",
    ] {
        textures.insert(
            texture_id.to_string(),
            load_texture(format!("assets/textures/{}.png", texture_id).as_str())
                .await
                .unwrap(),
        );
    }
    return textures;
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

        receive_game_state(&mut state);
        get_input(&mut state);
        udp_send_commands(&mut state);

        let input = &state.input;

        let highlighted_card_opt_clone = state.highlighted_card_opt.clone();
        state.highlighted_card_opt = None;
        for (i, card) in state.hand.hand.iter().enumerate() {
            let is_selected = highlighted_card_opt_clone == Some(i);
            let hovering: bool = draw_in_hand_card(
                card,
                i,
                state.hand.hand.len(),
                if is_selected { 0.5 } else { 1.0 },
                is_selected && !is_mouse_button_down(MouseButton::Left),
                state.relative_splay_radius,
                state.card_delta_angle,
                &textures,
                input.mouse_position,
            );
            if hovering {
                state.highlighted_card_opt = Some(i);
            }
        }

        if let Some(highlighted_card) = highlighted_card_opt_clone {
            if is_mouse_button_released(MouseButton::Left) {
                if input.mouse_in_world
                    && match state.hand.hand.get(highlighted_card).unwrap() {
                        Card::BasicTower => !input.mouse_over_occupied_tile,
                        Card::BasicRanger | Card::BasicDrone | Card::BasicUnit => true,
                    }
                {
                    if let Some(card) = state
                        .hand
                        .try_move_card_from_hand_to_played(highlighted_card)
                    {
                        state.commands.push(ClientCommand::PlayCard(
                            input.mouse_world_x,
                            input.mouse_world_y,
                            card.clone(),
                        ));
                    }
                }
                state.preview_tower_pos = None;
            } else {
                if is_mouse_button_down(MouseButton::Left) {
                    state.highlighted_card_opt = Some(highlighted_card);
                    if input.mouse_in_world {
                        match state.hand.hand.get(highlighted_card).unwrap() {
                            Card::BasicTower => {
                                state.preview_tower_pos = Some((
                                    input.mouse_world_x as i32 as f32 + 0.5,
                                    input.mouse_world_y as i32 as f32 + 0.5,
                                ));
                            }
                            Card::BasicRanger | Card::BasicDrone | Card::BasicUnit => {
                                draw_out_of_hand_card(
                                    state.hand.hand.get(highlighted_card).unwrap(),
                                    input.mouse_position.x,
                                    input.mouse_position.y,
                                    &textures,
                                    input.mouse_position,
                                );
                            }
                        }
                    } else {
                        draw_out_of_hand_card(
                            state.hand.hand.get(highlighted_card).unwrap(),
                            input.mouse_position.x,
                            input.mouse_position.y,
                            &textures,
                            input.mouse_position,
                        );
                    }
                } else {
                    let hovering = draw_highlighted_card(
                        state.hand.hand.get(highlighted_card).unwrap(),
                        highlighted_card,
                        state.relative_splay_radius,
                        state.card_delta_angle,
                        &textures,
                        input.mouse_position,
                        state.hand.hand.len(),
                    );
                    if hovering {
                        state.highlighted_card_opt = Some(highlighted_card);
                    }
                }
            }
        }
        next_frame().await
    }
}

pub fn main_draw(state: &mut ClientGameState) {
    clear_background(BLACK);

    //Draw board
    {
        for x in 0..state.static_game_state.grid_w {
            for y in 0..state.static_game_state.grid_h {
                draw_rectangle_ex(
                    u32_to_screen_x(x),
                    u32_to_screen_y(y),
                    cell_w(),
                    cell_h(),
                    DrawRectangleParams {
                        color: if state.static_game_state.path.contains(&(x as i32, y as i32)) {
                            PATH_COLOR
                        } else {
                            GRASS_COLOR
                        },
                        ..Default::default()
                    },
                );
            }
        }
        for (_id, entity) in state.dynamic_game_state.entities.iter() {
            let player = state.dynamic_game_state.players.get(&entity.owner);
            let color = if entity.damage_animation > 0.0 {
                RED
            } else {
                player.map_or(WHITE, |player| player.color)
            };
            match entity.tag {
                EntityTag::Tower => {
                    draw_hexagon(
                        f32_to_screen_x(entity.pos.x),
                        f32_to_screen_y(entity.pos.y),
                        20.0,
                        0.0,
                        false,
                        color,
                        color,
                    );
                }
                EntityTag::Unit => {
                    draw_circle(
                        f32_to_screen_x(entity.pos.x),
                        f32_to_screen_y(entity.pos.y),
                        to_screen_size(entity.radius),
                        color,
                    );
                }
                EntityTag::Drone => {
                    let rotation = if let Behavior::Drone(Drone {
                        target_entity_id, ..
                    }) = entity.behavior
                    {
                        if let Some(target_entity) = target_entity_id
                            .and_then(|id| state.dynamic_game_state.entities.get(&id))
                        {
                            (target_entity.pos - entity.pos).angle_between(Vec2::NEG_X)
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    draw_poly(
                        f32_to_screen_x(entity.pos.x),
                        f32_to_screen_y(entity.pos.y),
                        3,
                        to_screen_size(entity.radius),
                        360.0 * rotation,
                        color,
                    );
                }
                EntityTag::Bullet => {
                    draw_circle(
                        f32_to_screen_x(entity.pos.x),
                        f32_to_screen_y(entity.pos.y),
                        to_screen_size(PROJECTILE_RADIUS),
                        GRAY,
                    );
                }
            }
        }

        let mut range_circle_preview: Option<(f32, f32, f32, Color)> = None;
        if let Some((x, y)) = state.preview_tower_pos {
            if state.input.mouse_in_world {
                let color = if state.input.mouse_over_occupied_tile {
                    RED
                } else {
                    BLUE
                };
                draw_hexagon(
                    f32_to_screen_x(x),
                    f32_to_screen_y(y),
                    20.0,
                    0.0,
                    false,
                    GRAY,
                    Color { a: 0.5, ..color },
                );
                range_circle_preview = Some((x as f32, y as f32, 3.0, color));
            }
        } else if let Some(entity) = state
            .selected_entity_id
            .and_then(|id| state.dynamic_game_state.entities.get(&id))
        {
            if let Some(RangedAttack { range, .. }) = entity.ranged_attack {
                range_circle_preview = Some((entity.pos.x, entity.pos.y, range, BLUE));
            }
        }
        if let Some((x, y, range, color)) = range_circle_preview {
            draw_circle(
                f32_to_screen_x(x),
                f32_to_screen_y(y),
                to_screen_size(range),
                Color { a: 0.2, ..color },
            );
            draw_circle_lines(
                f32_to_screen_x(x),
                f32_to_screen_y(y),
                to_screen_size(range),
                2.0,
                color,
            );
        }
    }

    if let Some(player) = state.dynamic_game_state.players.get(&state.player_id) {
        let margin = 10.0;
        let outline_w = 5.0;
        let w = 25.0;
        let h = 100.0;
        let draw_progress = player.card_draw_counter;
        draw_progress_bar(
            screen_width() - w - margin,
            screen_height() - h - margin,
            w,
            h,
            outline_w,
            draw_progress,
            state.hand.hand.len() as i32,
            YELLOW,
            WHITE,
            BLACK,
        );
        let energy_progress = player.energy_counter;
        draw_progress_bar(
            screen_width() - 2.0 * w - 2.0 * margin,
            screen_height() - h - margin,
            w,
            h,
            outline_w,
            energy_progress,
            state.hand.energy,
            BLUE,
            WHITE,
            BLACK,
        );
    }
}
