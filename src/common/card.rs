use crate::{
    entity_blueprint::EntityBlueprint,
    ids::CardInstanceId,
    play_target::PlayFn,
    textures::SpriteId,
    world::{find_entity_mut, world_place_tower, world_place_unit},
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, EnumIter, Serialize, Deserialize, Eq, PartialEq)]
pub enum Card {
    Tower,
    SpawnPoint,
    HomesickWarrior,
    ElfWarrior,
    OldSwordMaster,
    DemonWolf,
    SmallCriminal,
    StreetCriminal,
    Spy,
    RecklessKnight,
    DirectDamage,
}

impl Card {
    pub fn iter() -> impl Iterator<Item = Card> {
        <Card as IntoEnumIterator>::iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CardInstance {
    pub id: CardInstanceId,
    pub card: Card,
}

pub struct CardData {
    pub name: &'static str,
    pub energy_cost: i32,
    pub sprite_id: SpriteId,
    pub play_fn: PlayFn,
}

macro_rules! play_building {
    ($builder_blueprint:ident, $building_blueprint:ident) => {
        PlayFn::BuildingSpot(|target, owner, static_game_state, dynamic_game_state| {
            world_place_tower(
                static_game_state,
                dynamic_game_state,
                target,
                owner,
                EntityBlueprint::$builder_blueprint,
                EntityBlueprint::$building_blueprint,
            )
        })
    };
}

macro_rules! play_unit {
    ($unit_blueprint:ident) => {
        PlayFn::UnitSpawnPoint(|target, owner, static_game_state, dynamic_game_state| {
            world_place_unit(
                static_game_state,
                dynamic_game_state,
                target,
                owner,
                EntityBlueprint::$unit_blueprint,
            )
        })
    };
}
const CARD_DATA: &[CardData] = &[
    CardData {
        name: "Tower",
        energy_cost: 3,
        sprite_id: SpriteId::CardTower,
        play_fn: play_building!(BasicBuilder, Tower),
    },
    CardData {
        name: "Spawn Point",
        energy_cost: 3,
        sprite_id: SpriteId::CardSpawnPoint,
        play_fn: play_building!(BasicBuilder, SpawnPoint),
    },
    CardData {
        name: "Homesick Warrior",
        energy_cost: 3,
        sprite_id: SpriteId::CardHomesickWarrior,
        play_fn: play_unit!(HomesickWarrior),
    },
    CardData {
        name: "Elf Warrior",
        energy_cost: 2,
        sprite_id: SpriteId::CardElfWarrior,
        play_fn: play_unit!(ElfWarrior),
    },
    CardData {
        name: "Old Sword Master",
        energy_cost: 3,
        sprite_id: SpriteId::CardOldSwordMaster,
        play_fn: play_unit!(OldSwordMaster),
    },
    CardData {
        name: "Demon Wolf",
        energy_cost: 3,
        sprite_id: SpriteId::CardDemonWolf,
        play_fn: play_unit!(DemonWolf),
    },
    CardData {
        name: "Small Criminal",
        energy_cost: 1,
        sprite_id: SpriteId::CardSmallCriminal,
        play_fn: play_unit!(SmallCriminal),
    },
    CardData {
        name: "Street Criminal",
        energy_cost: 2,
        sprite_id: SpriteId::CardStreetCriminal,
        play_fn: play_unit!(StreetCriminal),
    },
    CardData {
        name: "Spy",
        energy_cost: 3,
        sprite_id: SpriteId::CardSpy,
        play_fn: play_unit!(Spy),
    },
    CardData {
        name: "Reckless Knight",
        energy_cost: 2,
        sprite_id: SpriteId::CardRecklessKnight,
        play_fn: play_unit!(RecklessKnight),
    },
    CardData {
        name: "Direct Damage",
        energy_cost: 1,
        sprite_id: SpriteId::CardDirectDamage,
        play_fn: PlayFn::Entity(|target, _owner, _static_game_state, dynamic_game_state| {
            let Some(target_entity) =
                find_entity_mut(&mut dynamic_game_state.entities, Some(target.id))
            else {
                return false;
            };
            target_entity.health.deal_damage(150.0);
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
