use common::*;
use image::GenericImageView;
use macroquad::prelude::Vec2;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

fn main() -> std::io::Result<()> {
    let mut game_state = GameState::new();

    let img = image::open("path.png").unwrap();
    game_state.grid_w = img.dimensions().0;
    game_state.grid_h = img.dimensions().1;

    let is_path = |x: i32, y: i32| match (x.try_into(), y.try_into()) {
        (Ok(x), Ok(y)) if x < game_state.grid_w && y < game_state.grid_w => {
            img.get_pixel(x, y).0.get(0).is_some_and(|v| v > &0)
        }
        _ => false,
    };

    let path_start = (0..game_state.grid_w as i32)
        .into_iter()
        .flat_map(|x| (0..game_state.grid_w as i32).map(move |y| (x, y)))
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
    game_state.path = vec![(x, y)];
    while let Some(next) = vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
        .into_iter()
        .find_map(|next_xy| {
            (is_path(next_xy.0, next_xy.1) && !game_state.path.contains(&next_xy)).then(|| next_xy)
        })
    {
        game_state.path.push(next);
        (x, y) = next;
    }

    let udp_socket = UdpSocket::bind(SERVER_ADDR).unwrap();
    udp_socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    let mut next_unit_id = 0;
    let mut clients = Vec::<SocketAddr>::new();
    let mut time = SystemTime::now();
    loop {
        let old_time = time;
        time = SystemTime::now();
        let dt = time.duration_since(old_time).unwrap().as_secs_f32();
        println!("tick: {}, dt: {}ms", game_state.server_tick, dt * 1000.0);

        let client_message_buf = &mut [0; 50];
        let read_client_message = udp_socket.recv_from(client_message_buf);
        match read_client_message {
            Err(e) => match e.kind() {
                std::io::ErrorKind::ConnectionReset => {}
                std::io::ErrorKind::TimedOut => {}
                _ => {
                    dbg!(e);
                    panic!()
                }
            },
            Ok((amt, client_addr)) => {
                let command =
                    serde_json::from_slice::<ClientCommand>(&client_message_buf[..amt]).unwrap();
                dbg!(&command);
                match command {
                    ClientCommand::SpawnUnit(pos) => {
                        game_state.units.insert(
                            next_unit_id,
                            Unit {
                                pos,
                                speed: 1.0,
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
                    ClientCommand::JoinGame => {
                        if !clients.contains(&client_addr) {
                            clients.push(client_addr);
                        }
                    }
                }
            }
        }

        let msg = serde_json::to_string(&game_state).unwrap();
        for client in &clients {
            udp_socket.send_to(msg.as_bytes(), client).unwrap();
        }

        game_state.server_tick += 1;

        let mut units_to_remove = Vec::<u64>::new();
        for (id, unit) in game_state.units.iter_mut() {
            if unit.health <= 0.0 && !units_to_remove.contains(id) {
                units_to_remove.push(*id);
            } else {
                unit.pos += (unit.target - unit.pos).normalize_or_zero() * unit.speed * dt;
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
                        speed: 5.0,
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
        });
    }
}
