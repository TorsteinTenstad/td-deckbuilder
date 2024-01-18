use crate::{
    component_movement_behavior::{BulletMovementBehavior, MovementBehavior, MovementSpeed},
    config::PROJECTILE_RADIUS,
    entity::{Entity, EntityState, EntityTag},
    find_target::find_target_for_attack,
    outgoing_buff::{apply_buffs, OutgoingBuff, OutgoingBuffType},
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum AttackRange {
    Melee,
    Short,
    Default,
    Long,
    Custom(f32),
}

impl AttackRange {
    pub fn to_f32(&self, radius: f32) -> f32 {
        match self {
            AttackRange::Melee => radius,
            AttackRange::Short => 200.0,
            AttackRange::Default => 400.0,
            AttackRange::Long => 600.0,
            AttackRange::Custom(range) => *range,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AttackSpeed {
    Slow,
    Default,
    Fast,
    Custom(f32),
}

impl AttackSpeed {
    pub fn as_f32(&self) -> f32 {
        match self {
            AttackSpeed::Slow => 1.0,
            AttackSpeed::Default => 0.5,
            AttackSpeed::Fast => 0.25,
            AttackSpeed::Custom(speed) => *speed,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Attack {
    pub variant: AttackVariant,
    pub can_target: Vec<EntityTag>,
    range: AttackRange,
    damage: f32,
    attack_speed: AttackSpeed,
    pub cooldown_timer: f32,
    pub self_destruct: bool,
}

impl Attack {
    pub fn new(
        variant: AttackVariant,
        range: AttackRange,
        damage: f32,
        attack_interval: AttackSpeed,
        can_target: Vec<EntityTag>,
    ) -> Self {
        Self {
            variant,
            can_target,
            range,
            damage,
            attack_speed: attack_interval,
            cooldown_timer: 0.0,
            self_destruct: false,
        }
    }

    pub fn get_damage<'a, Iter>(&self, buffs: Iter) -> f32
    where
        Iter: Iterator<Item = &'a OutgoingBuff>,
    {
        apply_buffs(self.damage, buffs, OutgoingBuffType::AttackDamage)
    }

    pub fn get_range<'a, Iter>(&self, entity_radius: f32, buffs: Iter) -> f32
    where
        Iter: Iterator<Item = &'a OutgoingBuff>,
    {
        apply_buffs(
            self.range.to_f32(entity_radius),
            buffs,
            OutgoingBuffType::AttackRange,
        )
    }

    pub fn get_attack_speed<'a, Iter>(&self, buffs: Iter) -> f32
    where
        Iter: Iterator<Item = &'a OutgoingBuff>,
    {
        apply_buffs(
            self.attack_speed.as_f32(),
            buffs,
            OutgoingBuffType::AttackSpeed,
        )
    }
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum AttackVariant {
    Heal,
    RangedAttack,
    MeleeAttack,
}

impl Attack {
    pub fn update(entity: &mut Entity, entities: &mut Vec<Entity>, dt: f32) {
        // Clone to get around borrow checker:
        let entity_buffs = entity
            .buffs
            .iter()
            .map(|buff_instance| buff_instance.buff.clone())
            .collect::<Vec<_>>();
        for attack in &mut entity.attacks {
            let Some(target_entity) = find_target_for_attack(
                entity.pos,
                entity.owner,
                attack.get_range(entity.radius, entity_buffs.iter()),
                &attack,
                entities,
            ) else {
                continue;
            };
            if attack.cooldown_timer <= 0.0 {
                attack.cooldown_timer = attack.get_attack_speed(entity_buffs.iter());
                match attack.variant {
                    AttackVariant::RangedAttack => {
                        let mut bullet =
                            Entity::new(EntityTag::Bullet, entity.owner, EntityState::Moving);
                        bullet.pos = entity.pos;
                        bullet.movement_behavior =
                            MovementBehavior::Bullet(BulletMovementBehavior::new(
                                target_entity.id,
                                MovementSpeed::Projectile,
                            ));
                        bullet.radius = PROJECTILE_RADIUS;
                        bullet.health = 1.0;
                        bullet.hitbox_radius = PROJECTILE_RADIUS;
                        bullet.seconds_left_to_live = Some(3.0);
                        bullet.attacks.push(Attack {
                            variant: AttackVariant::MeleeAttack,
                            can_target: attack.can_target.clone(),
                            range: AttackRange::Melee,
                            damage: attack.damage,
                            attack_speed: AttackSpeed::Default,
                            cooldown_timer: 0.0,
                            self_destruct: true,
                        });

                        entities.push(bullet);
                    }
                    AttackVariant::MeleeAttack => {
                        target_entity.health -= attack.get_damage(entity_buffs.iter());
                        target_entity.damage_animation = 0.1;
                    }
                    AttackVariant::Heal => {
                        target_entity.health += attack.get_damage(entity_buffs.iter());
                        target_entity.health =
                            f32::min(target_entity.health, target_entity.max_health);
                    }
                }
                if attack.self_destruct {
                    entity.health = 0.0;
                };
            } else {
                attack.cooldown_timer -= dt;
            }
        }
    }
}
