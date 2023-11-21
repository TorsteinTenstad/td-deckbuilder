use std::collections::HashMap;

use common::{
    Entity, FreeKinematics, Kinematics, MeleeAttack, PathKinematics, RangedAttack, StaticGameState,
    StaticKinematics, PROJECTILE_RADIUS,
};
use macroquad::math::Vec2;

pub fn update_entity(
    id: &u64,
    entity: &Entity,
    other_entities: &HashMap<u64, Entity>,
    dt: f32,
    static_game_state: &StaticGameState,
    rng: &mut impl rand::Rng,
) -> Vec<(u64, Entity)> {
    let mut new_entities = Vec::new();
    let get_pos = |entity: &Entity| -> Vec2 {
        match entity.movement {
            Kinematics::Path(PathKinematics { path_pos, .. }) => {
                static_game_state.path_to_world_pos(path_pos)
            }
            Kinematics::Static(StaticKinematics { pos }) => pos,
            Kinematics::Free(FreeKinematics { pos, .. }) => pos,
        }
    };
    let entity_pos = get_pos(entity);
    let mut entity = entity.clone();

    match &mut entity.movement {
        Kinematics::Path(PathKinematics {
            path_pos,
            direction,
            speed,
        }) => {
            if !other_entities.iter().any(|(other_id, other_entity)| {
                id != other_id
                    && (get_pos(other_entity) - entity_pos).length_squared()
                        < (entity.radius + other_entity.radius).powi(2)
            }) {
                *path_pos += *speed * direction.to_f32() * dt;
            }
        }
        Kinematics::Static(StaticKinematics { pos: _ }) => {}
        Kinematics::Free(FreeKinematics {
            mut pos,
            mut velocity,
            target_entity_id,
            speed,
        }) => {
            velocity = target_entity_id
                .and_then(|target_entity_id| {
                    other_entities.get(&target_entity_id).map(|target_entity| {
                        (get_pos(target_entity) - pos).normalize_or_zero() * *speed
                    })
                })
                .unwrap_or(velocity);

            pos += velocity * dt;
        }
    };

    match entity.ranged_attack.as_mut() {
        Some(RangedAttack {
            range,
            damage,
            fire_rate,
            cooldown_timer,
        }) => {
            if *cooldown_timer <= 0.0 {
                if let Some((target_entity_id, _entity)) = other_entities
                    .iter()
                    .filter(|(_, entity)| entity.owner != entity.owner)
                    .map(|(id, entity)| (id, (entity_pos - get_pos(entity)).length_squared()))
                    .min_by(|(_, length_squared_a), (_, length_squared_b)| {
                        length_squared_a.partial_cmp(length_squared_b).unwrap()
                    })
                    .filter(|(_id, length_squared)| length_squared < &range.powi(2))
                {
                    *cooldown_timer = 1.0 / *fire_rate;
                    new_entities.push((
                        rng.gen::<u64>(),
                        Entity {
                            owner: entity.owner,
                            movement: Kinematics::Free(FreeKinematics {
                                pos: entity_pos,
                                velocity: Vec2::new(0.0, 0.0),
                                target_entity_id: Some(target_entity_id.clone()),
                                speed: 5.0,
                            }),
                            seconds_left_to_live: Some(3.0),
                            radius: PROJECTILE_RADIUS,
                            health: 1.0,
                            damage_animation: 0.0,
                            ranged_attack: None,
                            melee_attack: Some(MeleeAttack {
                                damage: *damage,
                                fire_rate: 0.5,
                                cooldown_timer: 0.0,
                            }),
                        },
                    ));
                }
            } else {
                *cooldown_timer -= dt;
            }
        }
        None => {}
    }

    entity.damage_animation -= dt;
    if let Some(seconds_left_to_live) = &mut entity.seconds_left_to_live {
        *seconds_left_to_live -= dt;
        if seconds_left_to_live < &mut 0.0 {
            entity.health = 0.0;
        }
    }
    if entity.health > 0.0 {
        new_entities.push((id.clone(), entity));
    }
    new_entities
}
