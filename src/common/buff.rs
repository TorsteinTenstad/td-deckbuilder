use serde::{Deserialize, Serialize};

use crate::entity::Entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticBuff {
    pub additive_value: f32,
    pub multiplier: f32,
    pub seconds_left: f32,
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
    pub seconds_left: f32,
}

#[derive(Clone, Serialize, Deserialize)]
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

pub fn buff_update_timers(entity: &mut Entity, dt: f32) {
    for attack in &mut entity.attacks {
        attack.damage_buffs.retain_mut(|buff| {
            buff.seconds_left -= dt;
            buff.seconds_left > 0.0
        });
        attack.attack_speed_buffs.retain_mut(|buff| {
            buff.seconds_left -= dt;
            buff.seconds_left > 0.0
        });
        attack.range_buffs.retain_mut(|buff| {
            buff.seconds_left -= dt;
            buff.seconds_left > 0.0
        });
    }
    if let Some(ref mut movement) = entity.movement {
        movement
            .movement_towards_target
            .speed_buffs
            .retain_mut(|buff| {
                buff.seconds_left -= dt;
                buff.seconds_left > 0.0
            });
    }
    entity.health.extra_health_buffs.retain_mut(|buff| {
        buff.seconds_left -= dt;
        buff.seconds_left > 0.0
    });
}
