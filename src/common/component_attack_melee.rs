use crate::entity::{Entity, EntityState, EntityTag};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MeleeAttack {
    pub can_target: Vec<EntityTag>,
    pub range: f32,
    pub damage: f32,
    pub attack_interval: f32,
    pub cooldown_timer: f32,
    pub die_on_hit: bool,
}

impl MeleeAttack {
    pub fn update(entity: &mut Entity, other_entities: &mut Vec<Entity>, dt: f32) {
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
                    .iter_mut()
                    .filter(|other_entity| other_entity.owner != entity.owner)
                    .filter(|other_entity| can_target.contains(&other_entity.tag))
                    .filter(|other_entity| {
                        (other_entity.pos - entity.pos).length_squared()
                            < (*range + other_entity.hitbox_radius).powi(2)
                    })
                    .min_by(|other_entity_a, other_entity_b| {
                        let signed_distance_a = (other_entity_a.pos - entity.pos).length_squared()
                            - (*range + other_entity_a.hitbox_radius).powi(2);
                        let signed_distance_b = (other_entity_b.pos - entity.pos).length_squared()
                            - (*range + other_entity_b.hitbox_radius).powi(2);
                        signed_distance_a.partial_cmp(&signed_distance_b).unwrap()
                    })
                {
                    entity.state = EntityState::Attacking;
                    if *cooldown_timer <= 0.0 {
                        *cooldown_timer = *attack_interval;
                        if *die_on_hit {
                            entity.health = 0.0;
                        };
                        target_entity.health -= *damage;
                        target_entity.damage_animation = 0.1;
                    } else {
                        *cooldown_timer -= dt;
                    }
                } else {
                    entity.state = EntityState::Moving;
                }
            }
            None => {}
        }
    }
}
