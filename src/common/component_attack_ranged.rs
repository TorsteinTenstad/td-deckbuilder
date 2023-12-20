use crate::entity::{Entity, EntityState, EntityTag};
use serde::{Deserialize, Serialize};

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
        entity: &mut Entity,
        other_entities: &mut Vec<Entity>,
        dt: f32,
        new_entities: &mut Vec<Entity>,
    ) {
        match entity.ranged_attack.as_mut() {
            Some(RangedAttack {
                can_target,
                range,
                damage,
                fire_interval,
                cooldown_timer,
            }) => {
                if let Some((target_entity_id, _entity)) = other_entities
                    .iter()
                    .filter(|other_entity| other_entity.owner != entity.owner)
                    .filter(|other_entity| can_target.contains(&other_entity.tag))
                    .map(|other_entity| {
                        (
                            other_entity.id,
                            (entity.pos - other_entity.pos).length_squared(),
                        )
                    })
                    .filter(|(_id, length_squared)| length_squared < &range.powi(2))
                    .min_by(|(_, length_squared_a), (_, length_squared_b)| {
                        length_squared_a.partial_cmp(length_squared_b).unwrap()
                    })
                {
                    entity.state = EntityState::Attacking;
                    if *cooldown_timer <= 0.0 {
                        *cooldown_timer = *fire_interval;
                        new_entities.push(Entity::new_bullet(
                            entity.owner,
                            entity.pos,
                            target_entity_id,
                            *damage,
                            300.0,
                        ));
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
