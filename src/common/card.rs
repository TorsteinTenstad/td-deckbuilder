use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Card {
    Tower,
    Unit,
}

impl Card {
    pub fn name(&self) -> &'static str {
        match self {
            Card::Tower => "Tower",
            Card::Unit => "Unit",
        }
    }
    pub fn energy_cost(&self) -> u32 {
        match self {
            Card::Tower => 5,
            Card::Unit => 3,
        }
    }
}
