use serde::{Deserialize, Serialize};

use crate::{
    play_target::{BuildingSpotTarget, PlayFn, UnitSpawnpointTarget},
    spawn_entity::spawn_entity,
    Entity,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Card {
    BasicTower,
    SpawnPointTest,
    BasicUnit,
    BasicRanger,
}

pub struct CardData {
    pub name: &'static str,
    pub energy_cost: i32,
    pub play_fn: PlayFn,
}

const CARD_DATA: &[CardData] = &[
    CardData {
        name: "Tower",
        energy_cost: 3,
        play_fn: PlayFn::BuildingSpot(|target, owner, static_game_state, dynamic_game_state| {
            let BuildingSpotTarget { id } = target;
            let (x, y) = static_game_state.building_locations.get(&id).unwrap();
            let entity = Entity::new_tower(owner, *x, *y, 3.0, 100.0, 2.0, 0.5);
            spawn_entity(dynamic_game_state, entity);
        }),
    },
    CardData {
        name: "Spawn Point",
        energy_cost: 2,
        play_fn: PlayFn::BuildingSpot(|target, owner, static_game_state, dynamic_game_state| {
            let BuildingSpotTarget { id } = target;
            let (x, y) = static_game_state.building_locations.get(&id).unwrap();
            let mut entity = Entity::new_tower(owner, *x, *y, 3.0, 100.0, 2.0, 5.0);
            entity.usable_as_spawn_point = true;
            spawn_entity(dynamic_game_state, entity);
        }),
    },
    CardData {
        name: "Ground Unit",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let UnitSpawnpointTarget {
                path_id,
                path_idx,
                direction,
            } = target;
            let entity = Entity::new_unit(
                static_game_state,
                owner,
                path_id,
                path_idx,
                direction,
                25.0,
                100.0,
                10.0,
                0.5,
                0.0,
                0.0,
                0.0,
            );
            spawn_entity(dynamic_game_state, entity);
        }),
    },
    CardData {
        name: "Ranger",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let UnitSpawnpointTarget {
                path_id,
                path_idx,
                direction,
            } = target;
            let entity = Entity::new_unit(
                static_game_state,
                owner,
                path_id,
                path_idx,
                direction,
                25.0,
                50.0,
                0.0,
                0.0,
                3.0,
                5.0,
                0.5,
            );
            spawn_entity(dynamic_game_state, entity);
        }),
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
