use common::*;
use local_ip_address::local_ip;
use macroquad::prelude::{
    clear_background, draw_circle, draw_hexagon, is_mouse_button_down, is_mouse_button_pressed,
    is_mouse_button_released, mouse_position, next_frame, screen_height, screen_width, Color,
    MouseButton, Vec2, BLACK, GRAY, RED, WHITE,
};
use macroquad::shapes::{draw_circle_lines, draw_rectangle_ex, DrawRectangleParams};
use macroquad::window::request_new_screen_size;
use std::net::UdpSocket;
use std::time::SystemTime;

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
                    std::io::ErrorKind::WouldBlock => {
                        break;
                    }
                    _ => {
                        dbg!(e);
                        panic!()
                    }
                },
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
        if is_mouse_button_pressed(MouseButton::Left) {
            commands.push(ClientCommand::SpawnUnit);
        }
        let preview_tower_pos =
            is_mouse_button_down(MouseButton::Right).then_some((mouse_world_x, mouse_world_y));
        if is_mouse_button_released(MouseButton::Left) {
            selected_tower = dynamic_game_state.towers.iter().find_map(|(id, tower)| {
                (tower.pos_x == mouse_world_x && tower.pos_y == mouse_world_y).then_some(id.clone())
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

        next_frame().await
    }
}
