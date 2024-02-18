use crate::{
    buff::{apply_arithmetic_buffs, ArithmeticBuff},
    component_movement::Movement,
    config::PROJECTILE_RADIUS,
    entity::{Entity, EntityState, EntityTag, Health},
    find_target::find_target_for_attack,
    game_state::{DynamicGameState, SemiStaticGameState, StaticGameState},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            AttackRange::Short => 75.0,
            AttackRange::Default => 150.0,
            AttackRange::Long => 225.0,
            AttackRange::Custom(range) => *range,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetPool {
    Enemies,
    Allies,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub damage: f32,
    pub attack_speed: AttackSpeed,
    pub range: AttackRange,
    pub variant: AttackVariant,
    pub target_pool: TargetPool,
    pub can_target: Vec<EntityTag>,
    pub cooldown_timer: f32,
    pub self_destruct: bool,
    pub damage_buffs: Vec<ArithmeticBuff>,
    pub attack_speed_buffs: Vec<ArithmeticBuff>,
    pub range_buffs: Vec<ArithmeticBuff>,
}

impl Default for Attack {
    fn default() -> Self {
        Self {
            damage: 0.0,
            attack_speed: AttackSpeed::Default,
            range: AttackRange::Melee,
            variant: AttackVariant::MeleeAttack,
            target_pool: TargetPool::Enemies,
            can_target: vec![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
            cooldown_timer: 0.0,
            self_destruct: false,
            damage_buffs: Vec::new(),
            attack_speed_buffs: Vec::new(),
            range_buffs: Vec::new(),
        }
    }
}

impl Attack {
    pub fn default_ranged() -> Self {
        Self {
            variant: AttackVariant::RangedAttack,
            range: AttackRange::Default,
            ..Default::default()
        }
    }
    pub fn default_ranged_tower() -> Self {
        Self {
            can_target: vec![EntityTag::Unit],
            ..Attack::default_ranged()
        }
    }
}

impl Attack {
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

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum AttackVariant {
    RangedAttack,
    MeleeAttack,
}

impl Attack {
    pub fn update(
        _static_game_state: &StaticGameState,
        _semi_static_game_state: &SemiStaticGameState,
        dynamic_game_state: &mut DynamicGameState,
        entity: &mut Entity,
        dt: f32,
    ) {
        for attack in &mut entity.attacks {
            let Some(target_entity) = find_target_for_attack(
                entity.id,
                entity.tag.clone(),
                entity.pos,
                entity.owner,
                entity.spy.as_ref(),
                attack.get_range(entity.radius),
                attack,
                &mut dynamic_game_state.entities,
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
                        bullet.movement = Some(Movement::new_projectile(target_entity.id));
                        bullet.radius = PROJECTILE_RADIUS;
                        bullet.health = Health::new(1.0);
                        bullet.hitbox_radius = PROJECTILE_RADIUS;
                        bullet.seconds_left_to_live = Some(3.0);

                        bullet.attacks.push(Attack {
                            damage: attack.get_damage(),
                            can_target: attack.can_target.clone(),
                            self_destruct: true,
                            ..Attack::default()
                        });

                        dynamic_game_state.entities.push(bullet);
                    }
                    AttackVariant::MeleeAttack => {
                        target_entity.health.deal_damage(attack.get_damage());
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
