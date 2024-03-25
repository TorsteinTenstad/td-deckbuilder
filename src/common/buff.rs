use serde::{Deserialize, Serialize};

use crate::entity::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticBuff {
    pub additive_value: f32,
    pub multiplier: f32,
    pub seconds_left: Option<f32>,
}

impl ArithmeticBuff {
    pub fn new_additive(additive_value: f32) -> Self {
        Self {
            additive_value,
            ..Default::default()
        }
    }
    pub fn new_multiplicative(multiplier: f32) -> Self {
        Self {
            multiplier,
            ..Default::default()
        }
    }
}

impl Default for ArithmeticBuff {
    fn default() -> Self {
        ArithmeticBuff {
            additive_value: 0.0,
            multiplier: 1.0,
            seconds_left: None,
        }
    }
}

pub fn apply_arithmetic_buffs(base_value: f32, buffs: &[ArithmeticBuff]) -> f32 {
    let mut additive_value = 0.0;
    let mut multiplier = 1.0;
    for buff in buffs {
        additive_value += buff.additive_value;
        multiplier *= buff.multiplier;
    }
    (base_value + additive_value) * multiplier
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraHealthBuff {
    pub health: f32,
    pub max_health: f32,
    pub seconds_left: Option<f32>,
}

impl ExtraHealthBuff {
    pub fn new(max_health: f32, seconds_left: Option<f32>) -> Self {
        Self {
            health: max_health,
            max_health,
            seconds_left,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Buff {
    AttackDamage(ArithmeticBuff),
    AttackSpeed(ArithmeticBuff),
    AttackRange(ArithmeticBuff),
    MovementSpeed(ArithmeticBuff),
    ExtraHealth(ExtraHealthBuff),
}

pub fn buff_add_to_entity(entity: &mut Entity, buff: Buff) {
    match buff {
        Buff::AttackDamage(buff) => {
            for attack in &mut entity.attacks {
                attack.damage_buffs.push(buff.clone());
            }
        }
        Buff::AttackSpeed(buff) => {
            for attack in &mut entity.attacks {
                attack.attack_speed_buffs.push(buff.clone());
            }
        }
        Buff::AttackRange(buff) => {
            for attack in &mut entity.attacks {
                attack.range_buffs.push(buff.clone());
            }
        }
        Buff::MovementSpeed(buff) => {
            if let Some(ref mut movement) = entity.movement {
                movement.movement_towards_target.speed_buffs.push(buff);
            }
        }
        Buff::ExtraHealth(buff) => {
            entity.health.extra_health_buffs.push(buff);
        }
    }
}

fn non_zero_after_update(seconds_left: &mut Option<f32>, dt: f32) -> bool {
    if let Some(ref mut seconds_left) = seconds_left {
        *seconds_left -= dt;
        *seconds_left > 0.0
    } else {
        false
    }
}

fn update_arithmetic_buffs(buffs: &mut Vec<ArithmeticBuff>, dt: f32) {
    buffs.retain_mut(|buff| non_zero_after_update(&mut buff.seconds_left, dt));
}

pub fn buff_update_timers(entity: &mut Entity, dt: f32) {
    for attack in &mut entity.attacks {
        update_arithmetic_buffs(&mut attack.damage_buffs, dt);
        update_arithmetic_buffs(&mut attack.attack_speed_buffs, dt);
        update_arithmetic_buffs(&mut attack.range_buffs, dt);
    }
    if let Some(ref mut movement) = entity.movement {
        update_arithmetic_buffs(&mut movement.movement_towards_target.speed_buffs, dt);
    }
    entity
        .health
        .extra_health_buffs
        .retain_mut(|buff| non_zero_after_update(&mut buff.seconds_left, dt));
}
