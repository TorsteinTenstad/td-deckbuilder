use serde::{Deserialize, Serialize};

use crate::{Entity, Player};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Card {
    BasicTower,
    BasicUnit,
    BasicSwarmer,
    BasicRanger,
}

impl Card {
    pub fn name(&self) -> &'static str {
        match self {
            Card::BasicTower => "Tower",
            Card::BasicUnit => "Ground Unit",
            Card::BasicSwarmer => "Swarmer",
            Card::BasicRanger => "Ranger",
        }
    }
    pub fn energy_cost(&self) -> i32 {
        match self {
            Card::BasicTower => 3,
            Card::BasicUnit => 1,
            Card::BasicSwarmer => 2,
            Card::BasicRanger => 1,
        }
    }

    pub fn to_entity(&self, player_id: u64, player: &Player, x: f32, y: f32) -> Entity {
        match self {
            Card::BasicTower => Entity::new_tower(player_id, x, y, 3.0, 100.0, 10.0, 2.0),
            Card::BasicUnit => Entity::new_unit(
                player_id,
                player.direction.clone(),
                1.0,
                100.0,
                10.0,
                2.0,
                0.0,
                0.0,
                0.0,
            ),
            Card::BasicSwarmer => Entity::new_swarmer(
                player_id,
                player.unit_start_pos.clone(),
                1.0,
                20.0,
                10.0,
                2.0,
            ),
            Card::BasicRanger => Entity::new_unit(
                player_id,
                player.direction.clone(),
                1.0,
                50.0,
                0.0,
                0.0,
                3.0,
                10.0,
                2.0,
            ),
        }
    }
}
