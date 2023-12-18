use std::collections::HashMap;

use common::{
    get_path_pos, next_path_idx, Behavior, BulletBehavior, Entity, EntityExternalEffects,
    MeleeAttack, PathUnitBehavior, RangedAttack, StaticGameState,
};

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
    let mut entity = entity.clone();

    match &mut entity.behavior {
        Behavior::PathUnit(PathUnitBehavior {
            path_id,
            target_path_idx,
            direction,
            speed,
        }) => {
            if *target_path_idx < static_game_state.paths.get(path_id).unwrap().len() {
                let target_pos = get_path_pos(static_game_state, *path_id, *target_path_idx);
                let delta = target_pos - entity.pos;
                entity.pos += delta.normalize_or_zero() * *speed * dt;
                let updated_delta = target_pos - entity.pos;
                if delta.length_squared() < updated_delta.length_squared() {
                    *target_path_idx = next_path_idx(*target_path_idx, *direction)
                }
            }
        }
        Behavior::Bullet(BulletBehavior {
            velocity,
            target_entity_id,
            speed,
        }) => {
            *velocity = target_entity_id
                .and_then(|target_entity_id| {
                    other_entities.get(&target_entity_id).map(|target_entity| {
                        (target_entity.pos - entity.pos).normalize_or_zero() * *speed
                    })
                })
                .unwrap_or(*velocity);

            entity.pos += *velocity * dt;
        }
        Behavior::None => {}
    };

    match entity.ranged_attack.as_mut() {
        Some(RangedAttack {
            can_target,
            range,
            damage,
            fire_interval,
            cooldown_timer,
        }) => {
            if *cooldown_timer <= 0.0 {
                if let Some((target_entity_id, _entity)) = other_entities
                    .iter()
                    .filter(|(_, other_entity)| other_entity.owner != entity.owner)
                    .filter(|(_, other_entity)| can_target.contains(&other_entity.tag))
                    .map(|(id, other_entity)| {
                        (id, (entity.pos - other_entity.pos).length_squared())
                    })
                    .filter(|(_id, length_squared)| length_squared < &range.powi(2))
                    .min_by(|(_, length_squared_a), (_, length_squared_b)| {
                        length_squared_a.partial_cmp(length_squared_b).unwrap()
                    })
                {
                    *cooldown_timer = *fire_interval;
                    new_entities.push((
                        rng.gen::<u64>(),
                        Entity::new_bullet(
                            entity.owner,
                            entity.pos,
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
            can_target,
            range,
            damage,
            attack_interval,
            cooldown_timer,
            die_on_hit,
        }) => {
            if let Some(target_entity) = other_entities
                .iter()
                .filter(|(_id, other_entity)| other_entity.owner != entity.owner)
                .filter(|(_id, other_entity)| can_target.contains(&other_entity.tag))
                .map(|(id, other_entity)| {
                    (
                        id,
                        (other_entity.pos - entity.pos).length_squared()
                            - (range.unwrap_or(entity.radius) + other_entity.radius).powi(2),
                    )
                })
                .filter(|(_id, signed_distance)| signed_distance < &0.0)
                .min_by(|(_id_a, signed_distance_a), (_id_b, signed_distance_b)| {
                    signed_distance_a.partial_cmp(signed_distance_b).unwrap()
                })
                .map(|(id, _signed_distance)| id)
            {
                if *cooldown_timer <= 0.0 {
                    *cooldown_timer = *attack_interval;
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
