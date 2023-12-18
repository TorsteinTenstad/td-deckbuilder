use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Entity, EntityExternalEffects, EntityTag, StaticGameState};

#[derive(Clone, Serialize, Deserialize)]
pub struct MeleeAttack {
    pub can_target: Vec<EntityTag>,
    pub range: Option<f32>,
    pub damage: f32,
    pub attack_interval: f32,
    pub cooldown_timer: f32,
    pub die_on_hit: bool,
}

impl MeleeAttack {
    pub fn update(
        id: &u64,
        entity: &mut Entity,
        other_entities: &HashMap<u64, Entity>,
        other_entities_external_effects: &mut HashMap<u64, EntityExternalEffects>,
        dt: f32,
        static_game_state: &StaticGameState,
        rng: &mut impl rand::Rng,
    ) {
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
    }
}
