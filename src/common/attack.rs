use crate::{
    entity::{Entity, EntityTag},
    world::find_entity_in_range,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Attack {
    pub variant: AttackVariant,
    pub can_target: Vec<EntityTag>,
    pub range: f32,
    pub damage: f32,
    pub attack_interval: f32,
    pub cooldown_timer: f32,
    pub self_destruct: bool,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum AttackVariant {
    RangedAttack,
    MeleeAttack,
}

impl Attack {
    pub fn update(
        entity: &mut Entity,
        other_entities: &mut Vec<Entity>,
        dt: f32,
        new_entities: &mut Vec<Entity>,
    ) {
        for attack in &mut entity.attacks {
            let Some(target_entity) = find_entity_in_range(
                entity.pos,
                entity.owner,
                attack.range,
                &attack.can_target,
                other_entities,
            ) else {
                continue;
            };
            if attack.cooldown_timer <= 0.0 {
                attack.cooldown_timer = attack.attack_interval;
                match attack.variant {
                    AttackVariant::RangedAttack => {
                        new_entities.push(Entity::new_bullet(
                            entity.owner,
                            entity.pos,
                            target_entity.id,
                            attack.damage,
                            300.0,
                            attack.can_target.clone(),
                        ));
                    }
                    AttackVariant::MeleeAttack => {
                        if attack.self_destruct {
                            entity.health = 0.0;
                        };
                        target_entity.health -= attack.damage;
                        target_entity.damage_animation = 0.1;
                    }
                }
            } else {
                attack.cooldown_timer -= dt;
            }
        }
    }
}
