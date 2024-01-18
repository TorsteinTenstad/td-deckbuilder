use std::mem;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct OutgoingBuffValue {
    pub additive_value: f32,
    pub multiplicative_value: f32,
}

impl OutgoingBuffValue {
    pub fn new_additive(value: f32) -> Self {
        Self {
            additive_value: value,
            ..Default::default()
        }
    }
    pub fn new_multiplicative(value: f32) -> Self {
        Self {
            multiplicative_value: value,
            ..Default::default()
        }
    }
    pub fn combine(&self, other: &OutgoingBuffValue) -> Self {
        Self {
            additive_value: self.additive_value + other.additive_value,
            multiplicative_value: self.multiplicative_value * other.multiplicative_value,
        }
    }
    pub fn apply(&self, base_value: f32) -> f32 {
        (base_value + self.additive_value) * self.multiplicative_value
    }
}

impl Default for OutgoingBuffValue {
    fn default() -> Self {
        Self {
            additive_value: 0.0,
            multiplicative_value: 1.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum OutgoingBuffType {
    AttackDamage,
    AttackRange,
    AttackSpeed,
    MovementSpeed,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OutgoingBuff {
    pub buff_type: OutgoingBuffType,
    pub buff_value: OutgoingBuffValue,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OutgoingBuffInstance {
    pub buff: OutgoingBuff,
    pub duration_left: f32,
}

pub fn apply_buffs<'a, Iter>(
    base_value: f32,
    buffs: Iter,
    buff_type_filter: OutgoingBuffType,
) -> f32
where
    Iter: Iterator<Item = &'a OutgoingBuff>,
{
    buffs
        .filter(|buff| mem::discriminant(&buff.buff_type) == mem::discriminant(&buff_type_filter))
        .fold(OutgoingBuffValue::default(), |a, b| {
            a.combine(&b.buff_value)
        })
        .apply(base_value)
}
