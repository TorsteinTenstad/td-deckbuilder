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

pub struct Cards {
    card_draw_counter: i32,
    energy_counter: i32,
    energy: i32,
    hand: Vec<Card>,
    deck: Vec<Card>,
    played: Vec<Card>,
}

impl Cards {
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

    pub fn try_move_card_form_hand_to_played(&mut self, index: usize) -> Option<Card> {
        if self.energy < self.hand.get(index).unwrap().energy_cost() {
            return None;
        }
        let card = self.hand.remove(index);
        self.energy -= card.energy_cost();
        self.played.push(card.clone());
        Some(card)
    }
}

#[macroquad::main("Client")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);
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

    let local_ip = local_ip().unwrap();

    let udp_socket = std::iter::successors(Some(6968), |port| Some(port + 1))
        .find_map(|port| {
            let socket_addr = SocketAddr::new(local_ip, port);
            UdpSocket::bind(socket_addr).ok()
        })
        .unwrap();
    udp_socket.set_nonblocking(true).unwrap();

    let send_join_game_command = || -> () {
        udp_socket
            .send_to(
                &serde_json::to_string(&ClientCommand::JoinGame)
                    .unwrap()
                    .as_bytes(),
                SERVER_ADDR,
            )
            .unwrap();
    };

    let mut static_game_state = StaticGameState::new();
    let mut dynamic_game_state = DynamicGameState::new();

    let mut time = SystemTime::now();
    let mut selected_entity_id: Option<u64> = None;

    let mut cards = Cards::new();
    let card_border = 5.0;
    let mut relative_splay_radius = 4.5;
    let mut card_delta_angle = 0.1;
    let card_visible_h = 0.8;

    let mut highlighted_card_opt: Option<usize> = None;
    let mut preview_tower_pos: Option<(f32, f32)> = None;

    let mut frames_since_last_received = 0;
    let mut commands = Vec::<ClientCommand>::new();
    let player_id = hash_client_addr(&udp_socket.local_addr().unwrap());
    loop {
        let old_time = time;
        time = SystemTime::now();
        let dt = time.duration_since(old_time).unwrap().as_secs_f32();
        // Receive game state
        {
            frames_since_last_received += 1;
            loop {
                let mut buf = [0; 20000];
                let received_message = udp_socket.recv_from(&mut buf);
                match received_message {
                    Ok((amt, _src)) => {
                        frames_since_last_received = 0;
                        let buf = &mut buf[..amt];
                        let log = |prefix: &str| {
                            let timestamp = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            std::fs::write(
                                format!("{}client_recv_{}.json", prefix, timestamp),
                                &buf,
                            )
                            .unwrap();
                        };
                        if is_key_down(macroquad::prelude::KeyCode::F11) {
                            log("");
                        }
                        let deserialization_result = serde_json::from_slice::<GameState>(buf); //TODO: handle error
                        if let Err(e) = deserialization_result {
                            log("error_");
                            dbg!(e);
                            panic!()
                        }
                        let received_game_state = deserialization_result.unwrap();
                        if received_game_state.dynamic_state.server_tick
                            > dynamic_game_state.server_tick
                            || received_game_state.static_state.game_id != static_game_state.game_id
                        {
                            if received_game_state.static_state.game_id != static_game_state.game_id
                            {
                                cards = Cards::new();
                            }
                            dynamic_game_state = received_game_state.dynamic_state;
                            static_game_state = received_game_state.static_state;
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
            if let Some(player) = dynamic_game_state.players.get(&player_id) {
                cards.sync_with_server_counters(
                    player.card_draw_counter as i32,
                    player.energy_counter as i32,
                );
            }
            if frames_since_last_received > 60 {
                send_join_game_command();
            }
        }

        let cell_w = screen_width() / static_game_state.grid_w as f32;
        let cell_h = (7.0 / 9.0) * screen_height() / static_game_state.grid_h as f32;
        let u32_to_screen_x = |x: u32| (x as f32) * cell_w;
        let u32_to_screen_y = |y: u32| (y as f32) * cell_h;
        let f32_to_screen_x = |x: f32| (x as f32) * cell_w;
        let f32_to_screen_y = |y: f32| (y as f32) * cell_h;
        let to_screen_size = |x: f32| x * cell_w;
        let (screen_space_mouse_x, screen_space_mouse_y) = mouse_position();
        let mouse_world_x = screen_space_mouse_x / cell_w;
        let mouse_world_y = screen_space_mouse_y / cell_h;
        let mouse_in_world = mouse_world_x >= 0.0
            && mouse_world_x >= 0.0
            && (mouse_world_x as u32) < static_game_state.grid_w
            && (mouse_world_y as u32) < static_game_state.grid_h;
        let mouse_over_occupied_tile = static_game_state
            .path
            .contains(&(mouse_world_x as i32, mouse_world_y as i32))
            || dynamic_game_state.entities.iter().any(|(_id, entity)| {
                entity.tag == EntityTag::Tower
                    && entity.pos.x as i32 == mouse_world_x as i32
                    && entity.pos.y as i32 == mouse_world_y as i32
            });

        clear_background(BLACK);

        if is_mouse_button_released(MouseButton::Left) {
            selected_entity_id = dynamic_game_state.entities.iter().find_map(|(id, entity)| {
                ((entity.pos
                    - Vec2 {
                        x: mouse_world_x,
                        y: mouse_world_y,
                    })
                .length()
                    < entity.radius)
                    .then(|| *id)
            });
        }
        //Send client commands
        {
            for command in &commands {
                udp_socket
                    .send_to(
                        &serde_json::to_string(&command).unwrap().as_bytes(),
                        SERVER_ADDR,
                    )
                    .unwrap();
            }
            commands.clear();
        }
        //Draw board
        {
            for x in 0..static_game_state.grid_w {
                for y in 0..static_game_state.grid_h {
                    draw_rectangle_ex(
                        u32_to_screen_x(x),
                        u32_to_screen_y(y),
                        cell_w,
                        cell_h,
                        DrawRectangleParams {
                            color: if static_game_state.path.contains(&(x as i32, y as i32)) {
                                PATH_COLOR
                            } else {
                                GRASS_COLOR
                            },
                            ..Default::default()
                        },
                    );
                }
            }
            for (_id, entity) in dynamic_game_state.entities.iter() {
                let player = dynamic_game_state.players.get(&entity.owner);
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
                        draw_poly(
                            f32_to_screen_x(entity.pos.x),
                            f32_to_screen_y(entity.pos.y),
                            3,
                            to_screen_size(entity.radius),
                            360.0
                                * if let Behavior::Drone(Drone {
                                    target_entity_id, ..
                                }) = entity.behavior
                                {
                                    if let Some(target_entity) = target_entity_id
                                        .and_then(|id| dynamic_game_state.entities.get(&id))
                                    {
                                        (target_entity.pos - entity.pos).angle_between(Vec2::NEG_X)
                                    } else {
                                        0.0
                                    }
                                } else {
                                    0.0
                                },
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
            if let Some((x, y)) = preview_tower_pos {
                if mouse_in_world {
                    let color = if mouse_over_occupied_tile { RED } else { BLUE };
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
            } else if let Some(entity) =
                selected_entity_id.and_then(|id| dynamic_game_state.entities.get(&id))
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

        if let Some(player) = dynamic_game_state.players.get(&player_id) {
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
                cards.hand.len() as i32,
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
                cards.energy,
                BLUE,
                WHITE,
                BLACK,
            );
        }

        //Card drawing parameter adjustment
        {
            if is_key_down(macroquad::prelude::KeyCode::L) {
                card_delta_angle += 0.05 * dt;
                dbg!(card_delta_angle);
            }
            if is_key_down(macroquad::prelude::KeyCode::J) {
                card_delta_angle -= 0.05 * dt;
                dbg!(card_delta_angle);
            }
            if is_key_down(macroquad::prelude::KeyCode::I) {
                relative_splay_radius += 0.5 * dt;
                dbg!(relative_splay_radius);
            }
            if is_key_down(macroquad::prelude::KeyCode::K) {
                relative_splay_radius -= 0.5 * dt;
                dbg!(relative_splay_radius);
            }
        }

        let mouse_position = Vec2::from_array(mouse_position().into());

        let draw_card =
            |card: &Card, x, y, w, h, rotation: f32, offset, alpha, only_compute_hover| -> bool {
                draw_circle(x, y, 3.0, YELLOW);
                let local_mouse_pos: Vec2 = Vec2::from_angle(-rotation)
                    .rotate(mouse_position - Vec2 { x, y })
                    + offset * Vec2 { x: w, y: h };
                let hovering = local_mouse_pos.cmpgt(Vec2::ZERO).all()
                    && local_mouse_pos.cmplt(Vec2::new(w, h)).all();
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
                        y: (2.0 * offset.y - 1.0) * card_border / h,
                    };
                draw_rectangle_ex(
                    x,
                    y,
                    w - 2.0 * card_border,
                    h - 2.0 * card_border,
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
                    &Player::new(Direction::Positive, Vec2::ZERO, BLACK),
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

                for (i, (texture_id, value)) in
                    icons.iter().filter(|(_, value)| *value > 0.001).enumerate()
                {
                    let width_relative_icon_size = 0.2;
                    let on_card_icon_pos = get_on_card_pos(
                        width_relative_margin,
                        2.0 * width_relative_margin + i as f32 * (width_relative_icon_size),
                    );
                    let on_card_value_pos = get_on_card_pos(
                        2.0 * width_relative_margin + width_relative_icon_size,
                        2.0 * width_relative_margin
                            + (i as f32 + 0.25) * (width_relative_icon_size),
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
            };
        let draw_in_hand_card_card = |card, i, n, alpha, only_compute_hover| -> bool {
            let card_w = screen_width() / 12.0;
            let card_h = card_w * GOLDEN_RATIO;
            let x = screen_width() / 2.0;
            let y = screen_height() + (relative_splay_radius * card_h) - (card_visible_h * card_h);
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
            )
        };

        let draw_out_of_hand_card = |card, x, y| -> bool {
            let card_w = screen_width() / 10.0;
            let card_h = card_w * GOLDEN_RATIO;
            draw_card(card, x, y, card_w, card_h, 0.0, 0.5 * Vec2::ONE, 1.0, false)
        };

        let draw_highlighted_card = |card, i| -> bool {
            let card_w = screen_width() / 10.0;
            let card_h = card_w * GOLDEN_RATIO;
            let x = screen_width() / 2.0
                + ((relative_splay_radius * card_h) - (card_visible_h * card_h))
                    * f32::sin(
                        (i as f32 - ((cards.hand.len() - 1) as f32 / 2.0)) * card_delta_angle,
                    );
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
            )
        };

        let highlighted_card_opt_clone = highlighted_card_opt.clone();
        highlighted_card_opt = None;
        for (i, card) in cards.hand.iter().enumerate() {
            let is_selected = highlighted_card_opt_clone == Some(i);
            let hovering = draw_in_hand_card_card(
                card,
                i,
                cards.hand.len(),
                if is_selected { 0.5 } else { 1.0 },
                is_selected && !is_mouse_button_down(MouseButton::Left),
            );
            if hovering {
                highlighted_card_opt = Some(i);
            }
        }
        if let Some(highlighted_card) = highlighted_card_opt_clone {
            if is_mouse_button_released(MouseButton::Left) {
                if mouse_in_world
                    && match cards.hand.get(highlighted_card).unwrap() {
                        Card::BasicTower => !mouse_over_occupied_tile,
                        Card::BasicRanger | Card::BasicDrone | Card::BasicUnit => true,
                    }
                {
                    if let Some(card) = cards.try_move_card_form_hand_to_played(highlighted_card) {
                        commands.push(ClientCommand::PlayCard(
                            mouse_world_x,
                            mouse_world_y,
                            card.clone(),
                        ));
                    }
                }
                preview_tower_pos = None;
            } else {
                if is_mouse_button_down(MouseButton::Left) {
                    highlighted_card_opt = Some(highlighted_card);
                    if mouse_in_world {
                        match cards.hand.get(highlighted_card).unwrap() {
                            Card::BasicTower => {
                                preview_tower_pos = Some((
                                    mouse_world_x as i32 as f32 + 0.5,
                                    mouse_world_y as i32 as f32 + 0.5,
                                ));
                            }
                            Card::BasicRanger | Card::BasicDrone | Card::BasicUnit => {
                                draw_out_of_hand_card(
                                    cards.hand.get(highlighted_card).unwrap(),
                                    mouse_position.x,
                                    mouse_position.y,
                                );
                            }
                        }
                    } else {
                        draw_out_of_hand_card(
                            cards.hand.get(highlighted_card).unwrap(),
                            mouse_position.x,
                            mouse_position.y,
                        );
                    }
                } else {
                    let hovering = draw_highlighted_card(
                        cards.hand.get(highlighted_card).unwrap(),
                        highlighted_card,
                    );
                    if hovering {
                        highlighted_card_opt = Some(highlighted_card);
                    }
                }
            }
        }
        next_frame().await
    }
}
