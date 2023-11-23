use serde::{Deserialize, Serialize};

use crate::{Entity, Player};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Card {
    BasicTower,
    BasicUnit,
    BasicDrone,
    BasicRanger,
}

impl Card {
    pub fn name(&self) -> &'static str {
        match self {
            Card::BasicTower => "Tower",
            Card::BasicUnit => "Ground Unit",
            Card::BasicDrone => "Drone",
            Card::BasicRanger => "Ranger",
        }
    }
    pub fn energy_cost(&self) -> i32 {
        match self {
            Card::BasicTower => 3,
            Card::BasicUnit => 1,
            Card::BasicDrone => 1,
            Card::BasicRanger => 1,
        }
    }

    pub fn to_entity(&self, player_id: u64, player: &Player, x: f32, y: f32) -> Entity {
        match self {
            Card::BasicTower => Entity::new_tower(player_id, x, y, 3.0, 100.0, 2.0, 0.5),
            Card::BasicUnit => Entity::new_unit(
                player_id,
                player.direction.clone(),
                1.0,
                100.0,
                10.0,
                0.5,
                0.0,
                0.0,
                0.0,
            ),
            Card::BasicDrone => Entity::new_drone(
                player_id,
                player.unit_start_pos.clone(),
                1.0,
                25.0,
                1.0,
                0.5,
            ),
            Card::BasicRanger => Entity::new_unit(
                player_id,
                player.direction.clone(),
                1.0,
                50.0,
                0.0,
                0.0,
                3.0,
                5.0,
                0.5,
            ),
        }
    }
}
