use serde::{Deserialize, Serialize};

use crate::{buff::Buff, update_args::UpdateArgs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffAura {
    buff: Buff,
    radius: Option<f32>,
    duration: Option<f32>,
}

impl BuffAura {
    pub fn update(_update_args: &mut UpdateArgs) {}
}
