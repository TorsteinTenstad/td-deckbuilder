use serde::{Deserialize, Serialize};

use crate::{Entity, Player};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Card {
    BasicTower,
    BasicGroundUnit,
    BasicSwarmer,
}

impl Card {
    pub fn name(&self) -> &'static str {
        match self {
            Card::BasicTower => "Tower",
            Card::BasicGroundUnit => "Ground Unit",
            Card::BasicSwarmer => "Swarmer",
        }
    }
    pub fn energy_cost(&self) -> u32 {
        match self {
            Card::BasicTower => 3,
            Card::BasicGroundUnit => 1,
            Card::BasicSwarmer => 2,
        }
    }

    pub fn to_entity(&self, player_id: u64, player: &Player, x: f32, y: f32) -> Entity {
        match self {
            Card::BasicTower => Entity::new_tower(player_id, x, y, 3.0, 100.0, 10.0, 2.0),
            Card::BasicGroundUnit => {
                Entity::new_unit(player_id, player.direction.clone(), 1.0, 100.0, 10.0, 2.0)
            }
            Card::BasicSwarmer => Entity::new_swarmer(
                player_id,
                player.unit_start_pos.clone(),
                1.0,
                20.0,
                10.0,
                2.0,
            ),
        }
    }
}
