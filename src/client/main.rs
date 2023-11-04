use common::*;
use local_ip_address::local_ip;
use macroquad::prelude::{
    clear_background, draw_circle, draw_hexagon, is_mouse_button_down, is_mouse_button_pressed,
    mouse_position, next_frame, screen_height, screen_width, Color, MouseButton, Vec2, BLACK, GRAY,
    GREEN, RED, WHITE,
};
use macroquad::shapes::{draw_rectangle, draw_rectangle_ex, DrawRectangleParams};
use macroquad::texture::DrawTextureParams;
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

    let mut game_state = GameState::new();

    let mut time = SystemTime::now();
    loop {
        let old_time = time;
        time = SystemTime::now();
        let dt = time.duration_since(old_time).unwrap().as_secs_f32();
        println!("tick: {}, dt: {}ms", game_state.server_tick, dt * 1000.0);
        loop {
            let mut buf = [0; 5000];
            let received_message = udp_socket.recv_from(&mut buf);
            match received_message {
                Ok((amt, _src)) => {
                    let buf = &mut buf[..amt];
                    game_state = serde_json::from_slice::<GameState>(buf).unwrap();
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
        let cell_w = screen_width() / game_state.grid_w as f32;
        let cell_h = screen_height() / game_state.grid_h as f32;
        let u32_to_screen_x = |x: u32| (x as f32 + 0.5) * cell_w;
        let u32_to_screen_y = |y: u32| (y as f32 + 0.5) * cell_h;
        let f32_to_screen_x = |x: f32| (x as f32 + 0.5) * cell_w;
        let f32_to_screen_y = |y: f32| (y as f32 + 0.5) * cell_h;
        let to_screen_size = |x: f32| x * cell_w;
        let (screen_space_mouse_x, screen_space_mouse_y) = mouse_position();
        let mouse_x = screen_space_mouse_x / cell_w - 0.5;
        let mouse_y = screen_space_mouse_y / cell_h - 0.5;

        let mut commands = Vec::<ClientCommand>::new();

        clear_background(BLACK);
        if is_mouse_button_pressed(MouseButton::Left) {
            commands.push(ClientCommand::SpawnUnit(Vec2::new(mouse_x, mouse_y)));
        }
        if is_mouse_button_down(MouseButton::Right) {
            commands.push(ClientCommand::SetTarget(Vec2::new(mouse_x, mouse_y)));
        }
        for command in commands {
            udp_socket
                .send_to(
                    &serde_json::to_string(&command).unwrap().as_bytes(),
                    SERVER_ADDR,
                )
                .unwrap();
        }
        for x in 0..game_state.grid_w {
            for y in 0..game_state.grid_h {
                draw_rectangle_ex(
                    u32_to_screen_x(x),
                    u32_to_screen_y(y),
                    cell_w,
                    cell_h,
                    DrawRectangleParams {
                        color: if game_state.path.contains(&(x as i32, y as i32)) {
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

        for (_id, unit) in game_state.units.iter() {
            draw_circle(
                f32_to_screen_x(unit.pos.x),
                f32_to_screen_y(unit.pos.y),
                to_screen_size(UNIT_RADIUS),
                if unit.damage_animation > 0.0 {
                    RED
                } else {
                    WHITE
                },
            );
        }

        for tower in game_state.towers.iter() {
            draw_hexagon(
                f32_to_screen_x(tower.pos.x),
                f32_to_screen_y(tower.pos.y),
                20.0,
                0.0,
                false,
                RED,
                RED,
            );
        }
        for projectile in &mut game_state.projectiles.iter() {
            draw_circle(
                f32_to_screen_x(projectile.pos.x),
                f32_to_screen_y(projectile.pos.y),
                to_screen_size(PROJECTILE_RADIUS),
                GRAY,
            );
        }

        draw_circle(
            f32_to_screen_x(game_state.target.x),
            f32_to_screen_y(game_state.target.y),
            4.0,
            GREEN,
        );

        next_frame().await
    }
}
