use serde::{Deserialize, Serialize};

use crate::{
    get_unit_spawnpoints::UnitSpawnpoint, spawn_entity::spawn_entity, Entity, PlayTarget,
    ServerGameState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Card {
    BasicTower,
    BasicUnit,
    BasicRanger,
}

pub struct CardData {
    pub name: &'static str,
    pub energy_cost: i32,
    pub play_fn: fn(u64, PlayTarget, &mut ServerGameState),
}

const CARD_DATA: &[CardData] = &[
    CardData {
        name: "Tower",
        energy_cost: 3,
        play_fn: |owner: u64, target, server_game_state: &mut ServerGameState| {
            let (x, y) = target.world_pos();
            let entity = Entity::new_tower(owner, x, y, 3.0, 100.0, 2.0, 0.5);
            spawn_entity(server_game_state, entity);
        },
    },
    CardData {
        name: "Ground Unit",
        energy_cost: 1,
        play_fn: |owner: u64, target, server_game_state: &mut ServerGameState| {
            let UnitSpawnpoint {
                path_id,
                direction,
                path_pos,
            } = target.unit_spawnpoint();
            let entity = Entity::new_unit(
                owner, path_id, direction, 1.0, 100.0, 10.0, 0.5, 0.0, 0.0, 0.0,
            );
            spawn_entity(server_game_state, entity);
        },
    },
    CardData {
        name: "Ranger",
        energy_cost: 1,
        play_fn: |owner: u64, target, server_game_state: &mut ServerGameState| {
            let UnitSpawnpoint {
                path_id,
                direction,
                path_pos,
            } = target.unit_spawnpoint();
            let entity = Entity::new_unit(
                owner, path_id, direction, 1.0, 50.0, 0.0, 0.0, 3.0, 5.0, 0.5,
            );
            spawn_entity(server_game_state, entity);
        },
    },
];

impl Card {
    pub fn get_card_data(&self) -> &CardData {
        CARD_DATA.get(self.clone() as usize).unwrap()
    }

    pub fn name(&self) -> &'static str {
        self.get_card_data().name
    }
    pub fn energy_cost(&self) -> i32 {
        self.get_card_data().energy_cost
    }
}
