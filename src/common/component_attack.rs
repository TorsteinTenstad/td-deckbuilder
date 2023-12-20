use crate::{
    component_movement_behavior::{BulletMovementBehavior, MovementBehavior},
    config::PROJECTILE_RADIUS,
    entity::{Entity, EntityState, EntityTag},
    world::find_entity_in_range,
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Attack {
    pub variant: AttackVariant,
    pub can_target: Option<Vec<EntityTag>>,
    pub range: f32,
    pub damage: f32,
    pub attack_interval: f32,
    pub cooldown_timer: f32,
    pub self_destruct: bool,
}

impl Attack {
    pub fn new(variant: AttackVariant, range: f32, damage: f32, attack_interval: f32) -> Self {
        Self {
            variant,
            can_target: None,
            range,
            damage,
            attack_interval,
            cooldown_timer: 0.0,
            self_destruct: false,
        }
    }
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
                        let mut bullet =
                            Entity::new(EntityTag::Bullet, entity.owner, EntityState::Moving);
                        bullet.pos = entity.pos;
                        bullet.movement_behavior =
                            MovementBehavior::Bullet(BulletMovementBehavior {
                                velocity: Vec2::ZERO,
                                target_entity_id: Some(target_entity.id),
                            });
                        bullet.radius = PROJECTILE_RADIUS;
                        bullet.health = 1.0;
                        bullet.hitbox_radius = PROJECTILE_RADIUS;
                        bullet.attacks.push(Attack::new(
                            AttackVariant::MeleeAttack,
                            PROJECTILE_RADIUS,
                            attack.damage,
                            0.0,
                        ));

                        new_entities.push(bullet);
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
