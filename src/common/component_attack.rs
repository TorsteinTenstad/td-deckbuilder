use crate::{
    buff::{apply_arithmetic_buffs, ArithmeticBuff},
    component_health::Health,
    component_movement::Movement,
    config::PROJECTILE_RADIUS,
    entities::Entities,
    entity::{Entity, EntityState, EntityTag},
    enum_flags::{flags, EnumFlags},
    find_target::{find_entities_in_range, find_target_id_for_attack},
    ids::{EntityId, PlayerId},
    update_args::UpdateArgs,
};
use macroquad::math::Vec2;
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
        let default_range = 100.0;
        match self {
            AttackRange::Melee => radius,
            AttackRange::Short => default_range / 1.5,
            AttackRange::Default => default_range,
            AttackRange::Long => default_range * 1.5,
            AttackRange::Custom(range) => *range,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackInterval {
    VerySlow,
    Slow,
    Default,
    Fast,
    VeryFast,
    Custom(f32),
}

impl AttackInterval {
    pub fn as_f32(&self) -> f32 {
        let default_speed = 0.5;
        match self {
            AttackInterval::VerySlow => default_speed / 2.0,
            AttackInterval::Slow => default_speed / 1.5,
            AttackInterval::Default => default_speed,
            AttackInterval::Fast => default_speed * 1.5,
            AttackInterval::VeryFast => default_speed * 2.0,
            AttackInterval::Custom(speed) => *speed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetPool {
    Enemies,
    Allies,
    All,
}

impl TargetPool {
    pub fn in_pool(&self, player_id_a: PlayerId, player_id_b: PlayerId) -> bool {
        match self {
            TargetPool::Allies => player_id_a == player_id_b,
            TargetPool::Enemies => player_id_a != player_id_b,
            TargetPool::All => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub damage: f32,
    pub attack_interval: AttackInterval,
    pub range: AttackRange,
    pub variant: AttackVariant,
    pub target_pool: TargetPool,
    pub can_target: EnumFlags<EntityTag>,
    pub cooldown_timer: f32,
    pub multi_attack_damage_range: Option<AttackRange>,
    pub self_destruct: bool,
    pub damage_buffs: Vec<ArithmeticBuff>,
    pub attack_speed_buffs: Vec<ArithmeticBuff>,
    pub range_buffs: Vec<ArithmeticBuff>,
}

impl Default for Attack {
    fn default() -> Self {
        Self {
            damage: 0.0,
            attack_interval: AttackInterval::Default,
            range: AttackRange::Melee,
            variant: AttackVariant::MeleeAttack,
            target_pool: TargetPool::Enemies,
            can_target: flags![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
            cooldown_timer: 0.0,
            self_destruct: false,
            multi_attack_damage_range: None,
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
            can_target: flags![
                EntityTag::Base,
                EntityTag::Tower,
                EntityTag::Unit,
                EntityTag::FlyingUnit,
            ],
            ..Default::default()
        }
    }
    pub fn default_flying() -> Self {
        Self {
            can_target: flags![EntityTag::Base, EntityTag::Tower, EntityTag::FlyingUnit],
            ..Attack::default()
        }
    }
    pub fn default_ranged_tower() -> Self {
        Self {
            can_target: flags![EntityTag::Unit, EntityTag::FlyingUnit],
            ..Attack::default_ranged()
        }
    }
}

impl Attack {
    pub fn get_damage(&self) -> f32 {
        apply_arithmetic_buffs(self.damage, &self.damage_buffs)
    }
    pub fn get_attack_interval(&self) -> f32 {
        let attack_speed = self.attack_interval.as_f32().recip();
        apply_arithmetic_buffs(attack_speed, &self.attack_speed_buffs).recip()
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
    pub fn update(update_args: &mut UpdateArgs) {
        for attack in &mut update_args.entity_instance.entity.attacks {
            let Some(target_id) = find_target_id_for_attack(
                update_args.entity_instance.id,
                update_args.entity_instance.entity.tag.clone(),
                update_args.entity_instance.pos,
                update_args.entity_instance.owner,
                update_args.entity_instance.entity.spy.as_ref(),
                attack.get_range(update_args.entity_instance.entity.radius),
                attack,
                &mut update_args.dynamic_game_state.entities,
            ) else {
                continue;
            };
            if attack.cooldown_timer <= 0.0 {
                attack.cooldown_timer = attack.get_attack_interval();
                attack.exec(
                    target_id,
                    update_args.entity_instance.pos,
                    update_args.entity_instance.owner,
                    &mut update_args.dynamic_game_state.entities,
                );
                if attack.self_destruct {
                    update_args.entity_instance.state = EntityState::Dead;
                };
            } else {
                attack.cooldown_timer -= update_args.dt;
            }
        }
    }
    fn exec(
        &mut self,
        target_id: EntityId,
        source_pos: Vec2,
        source_owner: PlayerId,
        entities: &mut Entities,
    ) {
        let target_ids = match self.multi_attack_damage_range.as_ref() {
            None => vec![target_id],
            Some(range) => find_entities_in_range(
                source_pos,
                range.to_f32(0.0), // TODO: fix this
                self.can_target.clone(),
                entities,
                |_| true,
            )
            .iter()
            .map(|e| e.id)
            .collect(),
        };
        for target_id in target_ids {
            match self.variant {
                AttackVariant::RangedAttack => {
                    let bullet = Entity {
                        tag: EntityTag::Bullet,
                        radius: PROJECTILE_RADIUS,
                        hitbox_radius: PROJECTILE_RADIUS,
                        health: Health::new(1.0),
                        movement: Some(Movement::new_projectile(target_id)),
                        seconds_left_to_live: Some(3.0),
                        attacks: vec![Attack {
                            damage: self.get_damage(),
                            can_target: self.can_target.clone(),
                            self_destruct: true,
                            ..Attack::default()
                        }],
                        ..Entity::default()
                    }
                    .instantiate(source_owner, source_pos);
                    entities.spawn(bullet);
                }
                AttackVariant::MeleeAttack => {
                    if let Some(target) = entities.iter_mut().find(|e| e.id == target_id) {
                        target.entity.health.deal_damage(self.get_damage());
                    } else {
                        debug_assert!(false);
                    }
                }
            }
        }
    }
}
