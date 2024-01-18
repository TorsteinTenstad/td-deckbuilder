use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Buff {
    pub additive_value: f32,
    pub multiplier: f32,
    pub seconds_left: f32,
}

pub fn apply_buffs(base_value: f32, buffs: &[Buff]) -> f32 {
    let mut additive_value = 0.0;
    let mut multiplier = 1.0;
    for buff in buffs {
        additive_value += buff.additive_value;
        multiplier *= buff.multiplier;
    }
    (base_value + additive_value) * multiplier
}
