use common::*;
use image::GenericImageView;
use local_ip_address::local_ip;
use macroquad::prelude::{
    clear_background, draw_circle, draw_hexagon, is_key_down, is_key_pressed, is_mouse_button_down,
    is_mouse_button_pressed, is_mouse_button_released, mouse_position, next_frame, screen_height,
    screen_width, Color, MouseButton, Vec2, BLACK, GRAY, LIGHTGRAY, RED, WHITE,
};
use macroquad::shapes::{
    draw_circle_lines, draw_rectangle_ex, draw_rectangle_lines, DrawRectangleParams,
};
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
    dbg!(&socket_addr);
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

    let mut time = SystemTime::now();
    let mut selected_tower: Option<u64> = None;

    let card_border = 5.0;
    let n = 5;
    let mut relative_splay_radius = 2.8;
    let mut card_delta_angle = 0.23;
    let card_visible_h = 0.8;

    let mut selected_card_opt: Option<i32> = None;

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
        let cell_h = screen_height() / static_game_state.grid_h as f32;
        let u32_to_screen_x = |x: u32| (x as f32 + 0.5) * cell_w;
        let u32_to_screen_y = |y: u32| (y as f32 + 0.5) * cell_h;
        let i32_to_screen_x = |x: i32| (x as f32 + 0.5) * cell_w;
        let i32_to_screen_y = |y: i32| (y as f32 + 0.5) * cell_h;
        let f32_to_screen_x = |x: f32| (x as f32 + 0.5) * cell_w;
        let f32_to_screen_y = |y: f32| (y as f32 + 0.5) * cell_h;
        let to_screen_size = |x: f32| x * cell_w;
        let (screen_space_mouse_x, screen_space_mouse_y) = mouse_position();
        let mouse_world_x = (screen_space_mouse_x / cell_w) as i32;
        let mouse_world_y = (screen_space_mouse_y / cell_h) as i32;

        let mut commands = Vec::<ClientCommand>::new();

        clear_background(BLACK);

        let mut preview_tower_pos: Option<(i32, i32)> = None;
        //Send client commands
        {
            if is_mouse_button_pressed(MouseButton::Left) {
                commands.push(ClientCommand::SpawnUnit);
            }
            preview_tower_pos =
                is_mouse_button_down(MouseButton::Right).then_some((mouse_world_x, mouse_world_y));
            if is_mouse_button_released(MouseButton::Left) {
                selected_tower = dynamic_game_state.towers.iter().find_map(|(id, tower)| {
                    (tower.pos_x == mouse_world_x && tower.pos_y == mouse_world_y)
                        .then_some(id.clone())
                });
            }
            if is_mouse_button_released(MouseButton::Right) {
                commands.push(ClientCommand::SpawnTower(mouse_world_x, mouse_world_y));
            }
            for command in commands {
                udp_socket
                    .send_to(
                        &serde_json::to_string(&command).unwrap().as_bytes(),
                        SERVER_ADDR,
                    )
                    .unwrap();
            }
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
                    RED,
                    RED,
                );
            }
            let mut range_circle_preview: Option<(i32, i32, f32)> = None;
            if let Some((x, y)) = preview_tower_pos {
                draw_hexagon(
                    i32_to_screen_x(x),
                    i32_to_screen_y(y),
                    20.0,
                    0.0,
                    false,
                    Color {
                        r: 0.5,
                        g: 0.5,
                        b: 0.5,
                        a: 0.5,
                    },
                    GRAY,
                );
                range_circle_preview = Some((x, y, 3.0));
            } else if let Some(id) = selected_tower {
                let tower = dynamic_game_state.towers.get(&id).unwrap();
                range_circle_preview = Some((tower.pos_x, tower.pos_y, tower.range));
            }
            if let Some((x, y, range)) = range_circle_preview {
                draw_circle(
                    i32_to_screen_x(x),
                    i32_to_screen_y(y),
                    to_screen_size(range),
                    Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.2,
                    },
                );
                draw_circle_lines(
                    i32_to_screen_x(x),
                    i32_to_screen_y(y),
                    to_screen_size(range),
                    2.0,
                    RED,
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

        let draw_card = |i: i32, selected: bool| -> bool {
            let selected_mask = if selected { 1.0 } else { 0.0 };
            let selected_size_modifier = 1.0 + 0.2 * selected_mask;
            let card_w = selected_size_modifier * screen_width() / 12.0;
            let card_h = card_w * GOLDEN_RATIO;
            let relative_splay_radius = relative_splay_radius / selected_size_modifier;
            let x = screen_width() / 2.0;
            let y = screen_height() + (relative_splay_radius * card_h) - (card_visible_h * card_h);
            let rotation = (i as f32 - ((n - 1) as f32 / 2.0)) * card_delta_angle;
            let mut offset = Vec2 {
                x: 0.5,
                y: relative_splay_radius,
            };
            let local_mouse_pos = Vec2::from_angle(-rotation)
                .rotate(mouse_position - Vec2 { x, y })
                + offset
                    * Vec2 {
                        x: card_w,
                        y: card_h,
                    };
            let hovering = local_mouse_pos.cmpgt(Vec2::ZERO).all()
                && local_mouse_pos.cmplt(Vec2::new(card_w, card_h)).all();

            draw_rectangle_ex(
                x,
                y,
                card_w,
                card_h,
                DrawRectangleParams {
                    color: if selected { RED } else { GRAY },
                    rotation,
                    offset,
                    ..Default::default()
                },
            );
            offset.y += (2.0 * relative_splay_radius - 1.0) * card_border / card_h;
            draw_rectangle_ex(
                x,
                y,
                card_w - 2.0 * card_border,
                card_h - 2.0 * card_border,
                DrawRectangleParams {
                    color: LIGHTGRAY,
                    rotation,
                    offset,
                    ..Default::default()
                },
            );
            hovering
        };
        let selected_card_opt_clone = selected_card_opt.clone();
        selected_card_opt = None;
        for i in 0..n {
            if selected_card_opt_clone == Some(i) {
                continue;
            }
            let hovering = draw_card(i, false);
            if hovering {
                selected_card_opt = Some(i);
            }
        }
        if let Some(selected_card) = selected_card_opt_clone {
            let hovering = draw_card(selected_card, true);
            if hovering {
                selected_card_opt = Some(selected_card);
            }
        }
        next_frame().await
    }
}
