use std::collections::HashMap;

use macroquad::prelude::*;

struct Unit {
    pos: Vec2,
    health: f32,
    damage_animation: f32,
    target: Vec2,
}

struct Tower {
    pos: Vec2,
    damage: f32,
    cooldown: f32,
    last_fire: f32,
}

struct Projectile {
    pos: Vec2,
    target_id: i32,
    speed: f32,
    velocity: Vec2,
    damage: f32,
}

const UNIT_RADIUS: f32 = 10.0;
const PROJECTILE_RADIUS: f32 = 4.0;

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut target = Vec2::new(100.0, 100.0);
    let mut next_available_unit_id = 0;
    let mut towers = vec![Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0)]
        .into_iter()
        .map(|pos| Tower {
            pos: pos,
            damage: 50.0,
            cooldown: 0.5,
            last_fire: 0.0,
        })
        .collect::<Vec<_>>();
    let mut units = HashMap::<i32, Unit>::new();
    let mut projectiles = Vec::<Projectile>::new();
    loop {
        clear_background(BLACK);

        if is_mouse_button_pressed(MouseButton::Left) {
            units.insert(
                next_available_unit_id,
                Unit {
                    pos: Vec2::from_array(mouse_position().into()),
                    health: 100.0,
                    damage_animation: 0.0,
                    target: target,
                },
            );
            next_available_unit_id += 1;
        }
        if is_mouse_button_down(MouseButton::Right) {
            target = Vec2::from_array(mouse_position().into());
            for (_id, unit) in units.iter_mut() {
                unit.target = target;
            }
        }

        let mut units_to_remove = Vec::<i32>::new();
        for (id, unit) in units.iter_mut() {
            if unit.health <= 0.0 && !units_to_remove.contains(id) {
                units_to_remove.push(*id);
            } else {
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
                unit.pos += (unit.target - unit.pos).normalize_or_zero() * 100.0 * get_frame_time();
                unit.damage_animation -= get_frame_time();
            }
        }
        for id in units_to_remove {
            units.remove(&id);
        }

        for tower in towers.iter_mut() {
            draw_hexagon(tower.pos.x, tower.pos.y, 20.0, 0.0, false, RED, RED);
            if tower.last_fire < 0.0 {
                if let Some((id, _unit)) = units.iter().min_by_key(|(_, unit)| {
                    (unit.pos - tower.pos).length();
                }) {
                    tower.last_fire = tower.cooldown;
                    projectiles.push(Projectile {
                        pos: tower.pos,
                        target_id: *id,
                        speed: 200.0,
                        velocity: Vec2::new(0.0, 0.0),
                        damage: tower.damage,
                    });
                }
            } else {
                tower.last_fire -= get_frame_time();
            }
        }
        let mut projectiles_to_remove = Vec::<usize>::new();
        for (i, projectile) in &mut projectiles.iter_mut().enumerate() {
            draw_circle(projectile.pos.x, projectile.pos.y, 4.0, GRAY);
            if let Some(target_unit) = units.get_mut(&projectile.target_id) {
                projectile.velocity =
                    (target_unit.pos - projectile.pos).normalize_or_zero() * projectile.speed;
            }
            projectile.pos += projectile.velocity * get_frame_time();
            for (_id, unit) in units.iter_mut() {
                if (unit.pos - projectile.pos).length() < UNIT_RADIUS + PROJECTILE_RADIUS {
                    unit.health -= projectile.damage;
                    unit.damage_animation = 0.05;
                    if !projectiles_to_remove.contains(&i) {
                        projectiles_to_remove.push(i);
                    }
                }
            }
        }
        for i in projectiles_to_remove {
            projectiles.remove(i);
        }

        draw_circle(target.x, target.y, 4.0, GREEN);

        next_frame().await
    }
}
