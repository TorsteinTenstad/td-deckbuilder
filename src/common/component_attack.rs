use crate::{
    buff::{apply_arithmetic_buffs, ArithmeticBuff},
    component_movement_behavior::{BulletMovementBehavior, MovementBehavior, MovementSpeed},
    config::PROJECTILE_RADIUS,
    entity::{Entity, EntityState, EntityTag, Health},
    find_target::find_target_for_attack,
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
    pub range: AttackRange,
    pub damage: f32,
    pub attack_speed: AttackSpeed,
    pub cooldown_timer: f32,
    pub self_destruct: bool,
    pub damage_buffs: Vec<ArithmeticBuff>,
    pub attack_speed_buffs: Vec<ArithmeticBuff>,
    pub range_buffs: Vec<ArithmeticBuff>,
}

impl Attack {
    pub fn new(
        variant: AttackVariant,
        range: AttackRange,
        damage: f32,
        attack_speed: AttackSpeed,
        can_target: Vec<EntityTag>,
    ) -> Self {
        Self {
            variant,
            can_target,
            range,
            damage,
            attack_speed,
            cooldown_timer: 0.0,
            self_destruct: false,
            damage_buffs: Vec::new(),
            attack_speed_buffs: Vec::new(),
            range_buffs: Vec::new(),
        }
    }
    pub fn get_damage(&self) -> f32 {
        apply_arithmetic_buffs(self.damage, &self.damage_buffs)
    }
    pub fn get_attack_speed(&self) -> f32 {
        apply_arithmetic_buffs(self.attack_speed.as_f32(), &self.attack_speed_buffs)
    }
    pub fn get_range(&self, radius: f32) -> f32 {
        apply_arithmetic_buffs(self.range.to_f32(radius), &self.range_buffs)
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
        for attack in &mut entity.attacks {
            let Some(target_entity) = find_target_for_attack(
                entity.pos,
                entity.owner,
                attack.get_range(entity.radius),
                &attack,
                entities,
            ) else {
                continue;
            };
            if attack.cooldown_timer <= 0.0 {
                attack.cooldown_timer = attack.get_attack_speed();
                match attack.variant {
                    AttackVariant::RangedAttack => {
                        let mut bullet =
                            Entity::new(EntityTag::Bullet, entity.owner, EntityState::Moving);
                        bullet.pos = entity.pos;
                        bullet.movement_behavior =
                            MovementBehavior::Bullet(BulletMovementBehavior {
                                speed: MovementSpeed::Projectile,
                                velocity: Vec2::ZERO,
                                target_entity_id: Some(target_entity.id),
                                speed_buffs: Vec::new(),
                            });
                        bullet.radius = PROJECTILE_RADIUS;
                        bullet.health = Health::new(1.0);
                        bullet.hitbox_radius = PROJECTILE_RADIUS;
                        bullet.seconds_left_to_live = Some(3.0);
                        let mut attack = Attack::new(
                            AttackVariant::MeleeAttack,
                            AttackRange::Melee,
                            attack.get_damage(),
                            AttackSpeed::Default,
                            attack.can_target.clone(),
                        );
                        attack.self_destruct = true;

                        bullet.attacks.push(attack);

                        entities.push(bullet);
                    }
                    AttackVariant::MeleeAttack => {
                        target_entity.health.deal_damage(attack.get_damage());
                    }
                    AttackVariant::Heal => {
                        target_entity.health.heal(attack.get_damage());
                    }
                }
                if attack.self_destruct {
                    entity.state = EntityState::Dead;
                };
            } else {
                attack.cooldown_timer -= dt;
            }
        }
    }
}
