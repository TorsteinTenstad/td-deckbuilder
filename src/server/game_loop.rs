use std::collections::HashMap;

use common::{
    Entity, EntityExternalEffects, FreeKinematics, Kinematics, MeleeAttack, PathKinematics,
    RangedAttack, StaticGameState, StaticKinematics,
};
use macroquad::math::Vec2;

pub fn update_entity(
    id: &u64,
    entity: &Entity,
    other_entities: &HashMap<u64, Entity>,
    other_entities_external_effects: &mut HashMap<u64, EntityExternalEffects>,
    dt: f32,
    static_game_state: &StaticGameState,
    rng: &mut impl rand::Rng,
) -> Vec<(u64, Entity)> {
    let mut new_entities = Vec::new();
    let get_pos = |entity: &Entity| -> Vec2 {
        match entity.kinematics {
            Kinematics::Path(PathKinematics { path_pos, .. }) => {
                static_game_state.path_to_world_pos(path_pos)
            }
            Kinematics::Static(StaticKinematics { pos }) => pos,
            Kinematics::Free(FreeKinematics { pos, .. }) => pos,
        }
    };
    let entity_pos = get_pos(entity);
    let mut entity = entity.clone();

    match &mut entity.kinematics {
        Kinematics::Path(PathKinematics {
            path_pos,
            direction,
            speed,
        }) => {
            if !other_entities.iter().any(|(other_id, other_entity)| {
                other_id != id
                    && matches!(
                        &other_entity.kinematics,
                        Kinematics::Path(PathKinematics {
                            path_pos: other_path_pos, ..
                        }) if {
                                let world_space_path_pos_delta =
                                 direction.to_f32()*(other_path_pos-
                                *path_pos )* static_game_state.path.len() as f32;
                                world_space_path_pos_delta > 0.0 && world_space_path_pos_delta < (other_entity.radius + entity.radius)
                        }
                    )
            }) {
                print!("{}\t", path_pos);
                *path_pos = (*path_pos * static_game_state.path.len() as f32
                + *speed * direction.to_f32() * dt)
                / static_game_state.path.len() as f32;
                println!("{}", path_pos);
            }
        }
        Kinematics::Static(StaticKinematics { pos: _ }) => {}
        Kinematics::Free(FreeKinematics {
            pos,
            velocity,
            target_entity_id,
            speed,
        }) => {
            *velocity = target_entity_id
                .and_then(|target_entity_id| {
                    other_entities.get(&target_entity_id).map(|target_entity| {
                        (get_pos(target_entity) - *pos).normalize_or_zero() * *speed
                    })
                })
                .unwrap_or(*velocity);

            *pos += *velocity * dt;
        }
    };

    match entity.ranged_attack.as_mut() {
        Some(RangedAttack {
            can_target,
            range,
            damage,
            fire_rate,
            cooldown_timer,
        }) => {
            if *cooldown_timer <= 0.0 {
                if let Some((target_entity_id, _entity)) = other_entities
                    .iter()
                    .filter(|(_, other_entity)| other_entity.owner != entity.owner)
                    .filter(|(_, other_entity)| {
                        other_entity
                            .tag
                            .as_ref()
                            .is_some_and(|tag| can_target.contains(&tag))
                    })
                    .map(|(id, other_entity)| {
                        (id, (entity_pos - get_pos(other_entity)).length_squared())
                    })
                    .filter(|(_id, length_squared)| length_squared < &range.powi(2))
                    .min_by(|(_, length_squared_a), (_, length_squared_b)| {
                        length_squared_a.partial_cmp(length_squared_b).unwrap()
                    })
                {
                    *cooldown_timer = 1.0 / *fire_rate;
                    new_entities.push((
                        rng.gen::<u64>(),
                        Entity::new_bullet(
                            entity.owner,
                            entity_pos,
                            *target_entity_id,
                            *damage,
                            5.0,
                        ),
                    ));
                }
            } else {
                *cooldown_timer -= dt;
            }
        }
        None => {}
    }

    match entity.melee_attack.as_mut() {
        Some(MeleeAttack {
            damage,
            fire_rate,
            cooldown_timer,
            die_on_hit,
        }) => {
            if let Some(target_entity) = other_entities
                .iter()
                .filter(|(_id, other_entity)| other_entity.owner != entity.owner)
                .map(|(id, other_entity)| {
                    (
                        id,
                        (get_pos(other_entity) - entity_pos).length_squared()
                            - (entity.radius + other_entity.radius).powi(2),
                    )
                })
                .filter(|(_id, signed_distance)| signed_distance < &0.0)
                .min_by(|(_id_a, signed_distance_a), (_id_b, signed_distance_b)| {
                    signed_distance_a.partial_cmp(signed_distance_b).unwrap()
                })
                .map(|(id, _signed_distance)| id)
            {
                if *cooldown_timer <= 0.0 {
                    *cooldown_timer = 1.0 / *fire_rate;
                    if *die_on_hit {
                        entity.health = 0.0;
                    };
                    if let Some(external_effects) =
                        other_entities_external_effects.get_mut(target_entity)
                    {
                        external_effects.health -= *damage;
                    } else {
                        other_entities_external_effects.insert(
                            target_entity.clone(),
                            EntityExternalEffects { health: -*damage },
                        );
                    }
                } else {
                    *cooldown_timer -= dt;
                }
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
