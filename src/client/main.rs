use common::*;
use local_ip_address::local_ip;
use macroquad::prelude::{
    clear_background, draw_circle, draw_hexagon, is_mouse_button_down, is_mouse_button_pressed,
    mouse_position, next_frame, screen_height, screen_width, MouseButton, Vec2, BLACK, GRAY, GREEN,
    RED, WHITE,
};
use macroquad::text::draw_text;
use macroquad::texture::{draw_texture_ex, load_texture, DrawTextureParams};
use macroquad::window::request_new_screen_size;
use serde_json::map::VacantEntry;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};
use std::{path, thread, time};

#[macroquad::main("BasicShapes")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);

    let path_texture = load_texture("path.png").await.unwrap();
    path_texture.set_filter(macroquad::texture::FilterMode::Nearest);
    let image = path_texture.get_texture_data();

    let is_path = |x: i32, y: i32| {
        x >= 0
            && y >= 0
            && x <= image.width as i32 - 1
            && y <= image.height as i32 - 1
            && image
                .get_pixel(x.try_into().unwrap(), y.try_into().unwrap())
                .r
                > 0.0
    };

    let path_start = (0..image.width as i32)
        .into_iter()
        .flat_map(|x| (0..image.height as i32).map(move |y| (x, y)))
        .find_map(|(x, y)| {
            (is_path(x, y)
                && (is_path(x - 1, y) as i32
                    + is_path(x, y - 1) as i32
                    + is_path(x + 1, y) as i32
                    + is_path(x, y + 1) as i32)
                    <= 1)
                .then(|| (x, y))
        });

    let (mut x, mut y) = path_start.unwrap();
    let mut path = vec![(x, y)];
    while let Some(next) = vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
        .into_iter()
        .find_map(|next_xy| {
            (is_path(next_xy.0, next_xy.1) && !path.contains(&next_xy)).then(|| next_xy)
        })
    {
        path.push(next);
        (x, y) = next;
    }
    loop {
        draw_texture_ex(
            &path_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        for (i, (x, y)) in path.iter().enumerate() {
            draw_text(
                format!("{}", i).as_str(),
                (*x as f32 + 0.5) * screen_width() / image.width as f32,
                (*y as f32 + 0.5) * screen_height() / image.height as f32,
                36.0,
                RED,
            );
        }
        next_frame().await;
    }
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
        let mut commands = Vec::<ClientCommand>::new();

        clear_background(BLACK);
        let (mouse_x, mouse_y) = mouse_position();
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

        for (_id, unit) in game_state.units.iter() {
            draw_circle(
                unit.pos.x,
                unit.pos.y,
                UNIT_RADIUS,
                if unit.damage_animation > 0.0 {
                    RED
                } else {
                    WHITE
                },
            );
        }

        for tower in game_state.towers.iter() {
            draw_hexagon(tower.pos.x, tower.pos.y, 20.0, 0.0, false, RED, RED);
        }
        for projectile in &mut game_state.projectiles.iter() {
            draw_circle(projectile.pos.x, projectile.pos.y, 4.0, GRAY);
        }

        draw_circle(game_state.target.x, game_state.target.y, 4.0, GREEN);

        next_frame().await
    }
}
