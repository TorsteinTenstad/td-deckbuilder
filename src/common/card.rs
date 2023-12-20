use crate::{
    component_movement_behavior::MovementBehavior,
    entity_blueprint::EntityBlueprint,
    ids::CardInstanceId,
    play_target::PlayFn,
    world::{get_path_pos, BuildingLocation},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Card {
    BasicTower,
    SpawnPointTest,
    BasicUnit,
    BasicRanger,
    DirectDamageTest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CardInstance {
    pub id: CardInstanceId,
    pub card: Card,
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
        play_fn: PlayFn::BuildingSpot(|target, owner, _static_game_state, dynamic_game_state| {
            let BuildingLocation { pos, entity_id } = dynamic_game_state
                .building_locations
                .get_mut(&target.id)
                .unwrap();
            if let Some(_) = entity_id {
                return false;
            }
            let mut entity = EntityBlueprint::BasicTower.create(owner);
            entity.pos = *pos;
            *entity_id = Some(entity.id);
            dynamic_game_state.entities.push(entity);
            return true;
        }),
    },
    CardData {
        name: "Spawn Point",
        energy_cost: 2,
        play_fn: PlayFn::BuildingSpot(|target, owner, _static_game_state, dynamic_game_state| {
            let BuildingLocation { pos, entity_id } = dynamic_game_state
                .building_locations
                .get_mut(&target.id)
                .unwrap();
            if let Some(_) = entity_id {
                return false;
            }
            let mut entity = EntityBlueprint::SpawnPointTest.create(owner);
            entity.pos = *pos;
            *entity_id = Some(entity.id);
            dynamic_game_state.entities.push(entity);
            return true;
        }),
    },
    CardData {
        name: "Ground Unit",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let mut entity = EntityBlueprint::BasicUnit.create(owner);
            entity.pos = get_path_pos(static_game_state, target.path_id, target.path_idx);
            entity.movement_behavior = MovementBehavior::Path(target.into());
            dynamic_game_state.entities.push(entity);
            return true;
        }),
    },
    CardData {
        name: "Ranger",
        energy_cost: 1,
        play_fn: PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            let mut entity = EntityBlueprint::BasicRanger.create(owner);
            entity.pos = get_path_pos(static_game_state, target.path_id, target.path_idx);
            entity.movement_behavior = MovementBehavior::Path(target.into());
            dynamic_game_state.entities.push(entity);
            return true;
        }),
    },
    CardData {
        name: "Direct Damage",
        energy_cost: 1,
        play_fn: PlayFn::Entity(|target, _owner, _static_game_state, dynamic_game_state| {
            let target_entity = dynamic_game_state
                .entities
                .iter_mut()
                .find(|entity| entity.id == target.id)
                .unwrap();
            target_entity.health -= 100.0;
            target_entity.damage_animation = 0.1;
            return true;
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
