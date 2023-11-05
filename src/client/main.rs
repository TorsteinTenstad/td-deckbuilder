use common::card::Card;
use common::*;
use local_ip_address::local_ip;
use macroquad::prelude::{
    clear_background, draw_circle, draw_hexagon, is_key_down, is_mouse_button_down,
    is_mouse_button_released, mouse_position, next_frame, screen_height, screen_width, Color,
    MouseButton, Vec2, BLACK, BLUE, GRAY, LIGHTGRAY, RED, WHITE,
};
use macroquad::shapes::{draw_circle_lines, draw_rectangle_ex, DrawRectangleParams};
use macroquad::window::request_new_screen_size;
use std::net::UdpSocket;
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

#[macroquad::main("Client")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);

    let local_ip = local_ip().unwrap();
    let socket_addr = format!("{}:34254", local_ip);
    let udp_socket = UdpSocket::bind(socket_addr).unwrap();
    udp_socket.set_nonblocking(true).unwrap();

    udp_socket
        .send_to(
            &serde_json::to_string(&ClientCommand::JoinGame)
                .unwrap()
                .as_bytes(),
            SERVER_ADDR,
        )
        .unwrap();

    let mut static_game_state = StaticGameState::new();
    let mut dynamic_game_state = DynamicGameState::new();
    let mut cards = vec![
        Card::Unit,
        Card::Unit,
        Card::Tower,
        Card::Tower,
        Card::Tower,
    ];

    let mut time = SystemTime::now();
    let mut selected_tower: Option<u64> = None;

    let card_border = 5.0;
    let mut relative_splay_radius = 2.8;
    let mut card_delta_angle = 0.23;
    let card_visible_h = 0.8;

    let mut highlighted_card_opt: Option<usize> = None;
    let mut preview_tower_pos: Option<(i32, i32)> = None;

    let mut commands = Vec::<ClientCommand>::new();

    loop {
        let old_time = time;
        time = SystemTime::now();
        let dt = time.duration_since(old_time).unwrap().as_secs_f32();
        if dt > 0.019 {
            println!(
                "tick: {}, dt: {}ms",
                dynamic_game_state.server_tick,
                dt * 1000.0
            );
        }
        // Receive game state
        {
            loop {
                let mut buf = [0; 5000];
                let received_message = udp_socket.recv_from(&mut buf);
                match received_message {
                    Ok((amt, _src)) => {
                        let buf = &mut buf[..amt];
                        let received_game_state = serde_json::from_slice::<GameState>(buf).unwrap();
                        if received_game_state.dynamic_state.server_tick
                            > dynamic_game_state.server_tick
                        {
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
        }

        let cell_w = screen_width() / static_game_state.grid_w as f32;
        let cell_h = (7.0 / 9.0) * screen_height() / static_game_state.grid_h as f32;
        let u32_to_screen_x = |x: u32| (x as f32 + 0.5) * cell_w;
        let u32_to_screen_y = |y: u32| (y as f32 + 0.5) * cell_h;
        let i32_to_screen_x = |x: i32| (x as f32 + 0.5) * cell_w;
        let i32_to_screen_y = |y: i32| (y as f32 + 0.5) * cell_h;
        let f32_to_screen_x = |x: f32| (x as f32 + 0.5) * cell_w;
        let f32_to_screen_y = |y: f32| (y as f32 + 0.5) * cell_h;
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
            || dynamic_game_state.towers.iter().any(|(_id, tower)| {
                tower.pos_x == mouse_world_x as i32 && tower.pos_y == mouse_world_y as i32
            });

        clear_background(BLACK);

        if is_mouse_button_released(MouseButton::Left) {
            selected_tower = dynamic_game_state.towers.iter().find_map(|(id, tower)| {
                (tower.pos_x == mouse_world_x as i32 && tower.pos_y == mouse_world_y as i32)
                    .then_some(id.clone())
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
                            offset: Vec2 { x: 0.5, y: 0.5 },
                            ..Default::default()
                        },
                    );
                }
            }

            for (_id, unit) in dynamic_game_state.units.iter() {
                let Vec2 {
                    x: world_x,
                    y: world_y,
                } = static_game_state.path_to_world_pos(unit.path_pos);
                draw_circle(
                    f32_to_screen_x(world_x),
                    f32_to_screen_y(world_y),
                    to_screen_size(UNIT_RADIUS),
                    if unit.damage_animation > 0.0 {
                        RED
                    } else {
                        WHITE
                    },
                );
            }

            for (_id, tower) in dynamic_game_state.towers.iter() {
                draw_hexagon(
                    i32_to_screen_x(tower.pos_x),
                    i32_to_screen_y(tower.pos_y),
                    20.0,
                    0.0,
                    false,
                    BLUE,
                    BLUE,
                );
            }
            let mut range_circle_preview: Option<(i32, i32, f32, Color)> = None;
            if let Some((x, y)) = preview_tower_pos {
                if mouse_in_world {
                    let color = if mouse_over_occupied_tile { RED } else { BLUE };
                    draw_hexagon(
                        i32_to_screen_x(x),
                        i32_to_screen_y(y),
                        20.0,
                        0.0,
                        false,
                        GRAY,
                        Color { a: 0.5, ..color },
                    );
                    range_circle_preview = Some((x, y, 3.0, color));
                }
            } else if let Some(id) = selected_tower {
                let tower = dynamic_game_state.towers.get(&id).unwrap();
                range_circle_preview = Some((tower.pos_x, tower.pos_y, tower.range, BLUE));
            }
            if let Some((x, y, range, color)) = range_circle_preview {
                draw_circle(
                    i32_to_screen_x(x),
                    i32_to_screen_y(y),
                    to_screen_size(range),
                    Color { a: 0.2, ..color },
                );
                draw_circle_lines(
                    i32_to_screen_x(x),
                    i32_to_screen_y(y),
                    to_screen_size(range),
                    2.0,
                    color,
                );
            }
            for projectile in &mut dynamic_game_state.projectiles.iter() {
                draw_circle(
                    f32_to_screen_x(projectile.pos.x),
                    f32_to_screen_y(projectile.pos.y),
                    to_screen_size(PROJECTILE_RADIUS),
                    GRAY,
                );
            }
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

        let draw_card = |x, y, w, h, rotation: f32, offset, alpha, only_compute_hover| -> bool {
            let mut offset = offset;
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
            offset.y += (2.0 * offset.y - 1.0) * card_border / h;
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
                    offset,
                    ..Default::default()
                },
            );
            hovering
        };
        let draw_in_hand_card_card = |i, n, alpha, only_compute_hover| -> bool {
            let card_w = screen_width() / 12.0;
            let card_h = card_w * GOLDEN_RATIO;
            let x = screen_width() / 2.0;
            let y = screen_height() + (relative_splay_radius * card_h) - (card_visible_h * card_h);
            let rotation = (i as f32 - ((cards.len() - 1) as f32 / 2.0)) * card_delta_angle;
            let offset = Vec2 {
                x: 0.5,
                y: relative_splay_radius,
            };
            draw_card(
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

        let draw_out_of_hand_card = |x, y| -> bool {
            let card_w = screen_width() / 10.0;
            let card_h = card_w * GOLDEN_RATIO;
            draw_card(x, y, card_w, card_h, 0.0, 0.5 * Vec2::ONE, 1.0, false)
        };

        let draw_highlighted_card = |i| -> bool {
            let card_w = screen_width() / 10.0;
            let card_h = card_w * GOLDEN_RATIO;
            let x = screen_width() / 2.0
                + ((relative_splay_radius * card_h) - (card_visible_h * card_h))
                    * f32::sin((i as f32 - ((cards.len() - 1) as f32 / 2.0)) * card_delta_angle);
            let y = screen_height();
            draw_card(
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
        for (i, _card) in cards.iter().enumerate() {
            let is_selected = highlighted_card_opt_clone == Some(i);
            let hovering = draw_in_hand_card_card(
                i,
                cards.len(),
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
                    && match cards.get(highlighted_card).unwrap() {
                        Card::Unit => true,
                        Card::Tower => !mouse_over_occupied_tile,
                    }
                {
                    commands.push(ClientCommand::PlayCard(
                        mouse_world_x,
                        mouse_world_y,
                        cards.get(highlighted_card).unwrap().clone(),
                    ));
                    cards.remove(highlighted_card);
                }
                preview_tower_pos = None;
            } else {
                if is_mouse_button_down(MouseButton::Left) {
                    highlighted_card_opt = Some(highlighted_card);
                    if mouse_in_world {
                        match cards.get(highlighted_card).unwrap() {
                            Card::Unit => {
                                draw_out_of_hand_card(mouse_position.x, mouse_position.y);
                            }
                            Card::Tower => {
                                preview_tower_pos =
                                    Some((mouse_world_x as i32, mouse_world_y as i32));
                            }
                        }
                    } else {
                        draw_out_of_hand_card(mouse_position.x, mouse_position.y);
                    }
                } else {
                    let hovering = draw_highlighted_card(highlighted_card);
                    if hovering {
                        highlighted_card_opt = Some(highlighted_card);
                    }
                }
            }
        }
        next_frame().await
    }
}
