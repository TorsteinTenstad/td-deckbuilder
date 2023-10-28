use common::*;
use macroquad::prelude::{
    clear_background, draw_circle, draw_hexagon, is_mouse_button_down, is_mouse_button_pressed,
    mouse_position, next_frame, MouseButton, Vec2, BLACK, GRAY, GREEN, RED, WHITE,
};
use std::net::UdpSocket;

#[macroquad::main("BasicShapes")]
async fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:34254").unwrap();

    loop {
        let mut buf = [0; 5000];
        let (amt, src) = udp_socket.recv_from(&mut buf).unwrap();
        let buf = &mut buf[..amt];
        let game_state = serde_json::from_slice::<GameState>(buf).unwrap();
        dbg!(game_state.server_tick);

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
                .send_to(&serde_json::to_string(&command).unwrap().as_bytes(), src)
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
