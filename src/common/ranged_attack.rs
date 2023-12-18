use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Entity, EntityExternalEffects, EntityTag, StaticGameState};

#[derive(Clone, Serialize, Deserialize)]
pub struct RangedAttack {
    pub can_target: Vec<EntityTag>,
    pub range: f32,
    pub damage: f32,
    pub fire_interval: f32,
    pub cooldown_timer: f32,
}

impl RangedAttack {
    pub fn update(
        id: &u64,
        entity: &mut Entity,
        other_entities: &HashMap<u64, Entity>,
        other_entities_external_effects: &mut HashMap<u64, EntityExternalEffects>,
        dt: f32,
        static_game_state: &StaticGameState,
        rng: &mut impl rand::Rng,
        new_entities: &mut Vec<(u64, Entity)>,
    ) {
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
    }
}
