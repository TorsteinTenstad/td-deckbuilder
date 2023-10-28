use common::*;
use macroquad::miniquad::fs;
use macroquad::prelude::Vec2;
use std::net::UdpSocket;
use std::time::Duration;
use std::{thread, time};

fn main() -> std::io::Result<()> {
    let mut game_state = GameState::new();
    let udp_socket = UdpSocket::bind("127.0.0.1:7878").unwrap();
    udp_socket.connect("127.0.0.1:34254").unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(15)))
        .unwrap();
    let mut next_unit_id = 0;

    loop {
        let dt = 0.016;
        game_state.server_tick += 1;
        let msg = serde_json::to_string(&game_state).unwrap();
        dbg!(msg.len());
        udp_socket
            .send_to(msg.as_bytes(), "127.0.0.1:34254")
            .unwrap();
        dbg!(game_state.server_tick);

        let buf = &mut [0; 50];
        let read_message = udp_socket.recv(buf);
        match read_message {
            Err(e) => match e.kind() {
                std::io::ErrorKind::TimedOut => {}
                _ => {
                    dbg!(e);
                    panic!()
                }
            },
            Ok(amt) => {
                let s = std::str::from_utf8(buf).unwrap();
                dbg!(s);
                let command = serde_json::from_slice::<ClientCommand>(&buf[..amt]).unwrap();
                match command {
                    ClientCommand::SpawnUnit(pos) => {
                        game_state.units.insert(
                            next_unit_id,
                            Unit {
                                pos,
                                target: game_state.target,
                                health: 100.0,
                                damage_animation: 0.0,
                            },
                        );
                        next_unit_id += 1;
                    }
                    ClientCommand::SetTarget(target) => {
                        game_state.target = target;
                    }
                }
            }
        }

        let mut units_to_remove = Vec::<u64>::new();
        for (id, unit) in game_state.units.iter_mut() {
            if unit.health <= 0.0 && !units_to_remove.contains(id) {
                units_to_remove.push(*id);
            } else {
                unit.pos += (unit.target - unit.pos).normalize_or_zero() * 100.0 * dt;
                unit.damage_animation -= dt;
            }
        }
        for id in units_to_remove {
            game_state.units.remove(&id);
        }

        for tower in game_state.towers.iter_mut() {
            if tower.last_fire < 0.0 {
                if let Some((id, _unit)) = game_state.units.iter().min_by_key(|(_, unit)| {
                    (unit.pos - tower.pos).length();
                }) {
                    tower.last_fire = tower.cooldown;
                    game_state.projectiles.push(Projectile {
                        pos: tower.pos,
                        target_id: *id,
                        speed: 500.0,
                        velocity: Vec2::new(0.0, 0.0),
                        damage: tower.damage,
                        seconds_left_to_live: 3.0,
                    });
                }
            } else {
                tower.last_fire -= dt;
            }
        }
        game_state.projectiles.retain_mut(|projectile| {
            if let Some(target_unit) = game_state.units.get_mut(&projectile.target_id) {
                projectile.velocity =
                    (target_unit.pos - projectile.pos).normalize_or_zero() * projectile.speed;
            }
            projectile.pos += projectile.velocity * dt;
            for (_id, unit) in game_state.units.iter_mut() {
                if (unit.pos - projectile.pos).length() < UNIT_RADIUS + PROJECTILE_RADIUS {
                    unit.health -= projectile.damage;
                    unit.damage_animation = 0.05;
                    return false;
                }
            }
            projectile.seconds_left_to_live -= dt;
            projectile.seconds_left_to_live > 0.0
        })
    }
}
