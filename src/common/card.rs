use crate::{
    buff::{buff_add_to_entity, ArithmeticBuff, Buff, ExtraHealthBuff},
    component_attack::AttackVariant,
    entity::{EntityState, EntityTag},
    entity_blueprint::EntityBlueprint,
    ids::CardInstanceId,
    level_config::get_prototype_level_config,
    play_target::{EntityTarget, PlayFn, SpecificPlayFn},
    world::{find_entity, find_entity_mut, world_place_builder, world_place_path_entity, Zoning},
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Hash, EnumIter, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Card {
    Tower,
    SmallTower,
    Watchtower,
    Wall,
    Farm,
    TradingPlace,
    IronMine,
    SpawnPoint,
    HomesickWarrior,
    ElfWarrior,
    OldSwordMaster,
    DemonWolf,
    SmallCriminal,
    StreetCriminal,
    Spy,
    RecklessKnight,
    Governor,
    WarEagle,
    AirBalloon,
    Dragon,
    DirectDamage,
    LightningStrike,
    ReinforcedDoors,
    HigherMotivation,
    SteadyAim,
    Meteor,
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
    pub play_fn: PlayFn,
    pub description: &'static str,
    pub card_art_path: &'static str,
    pub attack: Option<i32>,
    pub health: Option<i32>,
}

macro_rules! play_normal_building {
    ($builder_blueprint:ident, $building_blueprint:ident) => {
        PlayFn::BuildingLocation(
            SpecificPlayFn::new(
                |target, owner, static_game_state, semi_static_game_state, dynamic_game_state| {
                    world_place_builder(
                        static_game_state,
                        semi_static_game_state,
                        dynamic_game_state,
                        target,
                        EntityBlueprint::$builder_blueprint.create(),
                        EntityBlueprint::$building_blueprint,
                        owner,
                    )
                },
            )
            .with_target_is_invalid(
                |target,
                 _owner,
                 _static_game_state,
                 semi_static_game_state,
                 _dynamic_game_state| {
                    semi_static_game_state
                        .building_locations()
                        .get(&target.id)
                        .unwrap()
                        .zoning
                        != Zoning::Normal
                },
            ),
        )
    };
}

macro_rules! play_commerce_building {
    ($builder_blueprint:ident, $building_blueprint:ident) => {
        PlayFn::BuildingLocation(
            SpecificPlayFn::new(
                |target, owner, static_game_state, semi_static_game_state, dynamic_game_state| {
                    world_place_builder(
                        static_game_state,
                        semi_static_game_state,
                        dynamic_game_state,
                        target,
                        EntityBlueprint::$builder_blueprint.create(),
                        EntityBlueprint::$building_blueprint,
                        owner,
                    )
                },
            )
            .with_target_is_invalid(
                |target,
                 _owner,
                 _static_game_state,
                 semi_static_game_state,
                 _dynamic_game_state| {
                    semi_static_game_state
                        .building_locations()
                        .get(&target.id)
                        .unwrap()
                        .zoning
                        != Zoning::Commerce
                },
            ),
        )
    };
}

macro_rules! play_unit {
    ($unit_blueprint:ident) => {
        PlayFn::UnitSpawnPoint(SpecificPlayFn::new(
            |target, owner, static_game_state, _semi_static_game_state, dynamic_game_state| {
                world_place_path_entity(
                    static_game_state,
                    dynamic_game_state,
                    target,
                    EntityBlueprint::$unit_blueprint.create(),
                    owner,
                )
                .is_some()
            },
        ))
    };
}

impl Card {
    pub fn get_card_data(&self) -> CardData {
        match self {
            Card::Tower => CardData {
                name: "Tower",
                energy_cost: 3,
                play_fn: play_normal_building!(BasicBuilder, Tower),
                card_art_path: "tower.jpg",
                attack: EntityBlueprint::Tower.get_attack(),
                health: EntityBlueprint::Tower.get_health(),
                description: "[Ranged]",
            },
            Card::SmallTower => CardData {
                name: "Small Tower",
                energy_cost: 2,
                play_fn: play_normal_building!(BasicBuilder, SmallTower),
                card_art_path: "small_tower.jpg",
                attack: EntityBlueprint::SmallTower.get_attack(),
                health: EntityBlueprint::SmallTower.get_health(),
                description: "[Ranged] Gets 20% higher attack speed for each nearby tower",
            },
            Card::Watchtower => CardData {
                name: "Watchtower",
                energy_cost: 5,
                play_fn: play_normal_building!(BasicBuilder, Watchtower),
                card_art_path: "watchtower.jpg",
                attack: EntityBlueprint::Watchtower.get_attack(),
                health: EntityBlueprint::Watchtower.get_health(),
                description: "[Ranged] Nearby ranged attacks have double range",
            },
            Card::Wall => CardData {
                name: "Wall",
                energy_cost: 2,
                play_fn: play_normal_building!(BasicBuilder, Wall),
                card_art_path: "wall.jpg",
                attack: EntityBlueprint::Wall.get_attack(),
                health: EntityBlueprint::Wall.get_health(),
                description: "",
            },
            Card::Farm => CardData {
                name: "Farm",
                energy_cost: 4,
                play_fn: play_commerce_building!(BasicBuilder, Farm),
                card_art_path: "farm.jpg",
                attack: EntityBlueprint::Farm.get_attack(),
                health: EntityBlueprint::Farm.get_health(),
                description: "Increases drawing speed by 40%",
            },
            Card::TradingPlace => CardData {
                name: "Trading Place",
                energy_cost: 4,
                play_fn: play_commerce_building!(BasicBuilder, TradingPlace),
                card_art_path: "trading_place.jpg",
                attack: EntityBlueprint::TradingPlace.get_attack(),
                health: EntityBlueprint::TradingPlace.get_health(),
                description: "Increases energy generation by 40%",
            },
            Card::IronMine => CardData {
                name: "Iron Mine",
                energy_cost: 4,
                play_fn: play_commerce_building!(BasicBuilder, IronMine),
                card_art_path: "iron_mine.jpg",
                attack: EntityBlueprint::IronMine.get_attack(),
                health: EntityBlueprint::IronMine.get_health(),
                description: "Towers get +500 health when they are built",
            },
            Card::SpawnPoint => CardData {
                name: "Spawn Point",
                energy_cost: 3,
                play_fn: play_normal_building!(BasicBuilder, SpawnPoint),
                card_art_path: "spawn_point.jpg",
                attack: EntityBlueprint::SpawnPoint.get_attack(),
                health: EntityBlueprint::SpawnPoint.get_health(),
                description: "You may spawn units\nfrom this building",
            },
            Card::HomesickWarrior => CardData {
                name: "Homesick Warrior",
                energy_cost: 3,
                play_fn: play_unit!(HomesickWarrior),
                card_art_path: "homesick_warrior.jpg",
                attack: EntityBlueprint::HomesickWarrior.get_attack(),
                health: EntityBlueprint::HomesickWarrior.get_health(),
                description: "[Protector]",
            },
            Card::ElfWarrior => CardData {
                name: "Elf Warrior",
                energy_cost: 2,
                play_fn: play_unit!(ElfWarrior),
                card_art_path: "elf_warrior.jpg",
                attack: EntityBlueprint::ElfWarrior.get_attack(),
                health: EntityBlueprint::ElfWarrior.get_health(),
                description: "[Fast attacking], [Ranged]",
            },
            Card::OldSwordMaster => CardData {
                name: "Old Sword Master",
                energy_cost: 4,
                play_fn: play_unit!(OldSwordMaster),
                card_art_path: "old_sword_master.jpg",
                attack: EntityBlueprint::OldSwordMaster.get_attack(),
                health: EntityBlueprint::OldSwordMaster.get_health(),
                description: "[Very slow moving]",
            },
            Card::DemonWolf => CardData {
                name: "Demon Wolf",
                energy_cost: 3,
                play_fn: play_unit!(DemonWolf),
                card_art_path: "demon_wolf.jpg",
                attack: EntityBlueprint::DemonWolf.get_attack(),
                health: EntityBlueprint::DemonWolf.get_health(),
                description: "[Fast moving]",
            },
            Card::SmallCriminal => CardData {
                name: "Small Criminal",
                energy_cost: 1,
                play_fn: play_unit!(SmallCriminal),
                card_art_path: "small_criminal.jpg",
                attack: EntityBlueprint::SmallCriminal.get_attack(),
                health: EntityBlueprint::SmallCriminal.get_health(),
                description: "[Fast moving]",
            },
            Card::StreetCriminal => CardData {
                name: "Street Criminal",
                energy_cost: 2,
                play_fn: play_unit!(StreetCriminal),
                card_art_path: "street_criminal.jpg",
                attack: EntityBlueprint::StreetCriminal.get_attack(),
                health: EntityBlueprint::StreetCriminal.get_health(),
                description: "[Fast attacking]",
            },
            Card::Spy => CardData {
                name: "Spy",
                energy_cost: 3,
                play_fn: play_unit!(Spy),
                card_art_path: "spy.jpg",
                attack: EntityBlueprint::Spy.get_attack(),
                health: EntityBlueprint::Spy.get_health(),
                description: "Will not be seen\nby the first\n2 enemies it passes",
            },
            Card::RecklessKnight => CardData {
                name: "Reckless Knight",
                energy_cost: 2,
                play_fn: play_unit!(RecklessKnight),
                card_art_path: "reckless_knight.jpg",
                attack: EntityBlueprint::RecklessKnight.get_attack(),
                health: EntityBlueprint::RecklessKnight.get_health(),
                description: "[Fast moving]",
            },
            Card::Governor => CardData {
                name: "Governor",
                energy_cost: 4,
                play_fn: play_unit!(Governor),
                card_art_path: "governor.jpg",
                attack: EntityBlueprint::Governor.get_attack(),
                health: EntityBlueprint::Governor.get_health(),
                description: "Deals 5 additional damage\nfor each tower\nyou control",
            },
            Card::WarEagle => CardData {
                name: "War Eagle",
                energy_cost: 3,
                play_fn: play_unit!(WarEagle),
                card_art_path: "war_eagle.jpg",
                attack: EntityBlueprint::WarEagle.get_attack(),
                health: EntityBlueprint::WarEagle.get_health(),
                description: "[Flying]",
            },
            Card::AirBalloon => CardData {
                name: "Air Balloon",
                energy_cost: 5,
                play_fn: play_unit!(AirBalloon),
                card_art_path: "air_balloon.jpg",
                attack: EntityBlueprint::AirBalloon.get_attack(),
                health: EntityBlueprint::AirBalloon.get_health(),
                description: "[Flying]",
            },
            Card::Dragon => CardData {
                name: "Dragon",
                energy_cost: 7,
                play_fn: play_unit!(Dragon),
                card_art_path: "dragon.jpg",
                attack: EntityBlueprint::Dragon.get_attack(),
                health: EntityBlueprint::Dragon.get_health(),
                description: "[Flying]",
            },
            Card::DirectDamage => CardData {
                name: "Direct Damage",
                energy_cost: 1,
                play_fn: PlayFn::Entity(SpecificPlayFn::new(
                    |target,
                     _owner,
                     _static_game_state,
                     _semi_static_game_state,
                     dynamic_game_state| {
                        let Some(target_entity_instance) =
                            find_entity_mut(&mut dynamic_game_state.entities, Some(target.id))
                        else {
                            return false;
                        };
                        target_entity_instance.entity.health.deal_damage(150.0);
                        true
                    },
                )),
                card_art_path: "direct_damage.jpg",
                attack: None,
                health: None,
                description: "Deal 150 damage\nto a single unit or building",
            },
            Card::LightningStrike => CardData {
                name: "Lightning Strike",
                energy_cost: 3,
                play_fn: PlayFn::WorldPos(SpecificPlayFn::new(
                    |target,
                     _owner,
                     _static_game_state,
                     _semi_static_game_state,
                     dynamic_game_state| {
                        for entity_instance in dynamic_game_state.entities.iter_mut() {
                            if entity_instance.pos.distance(target.to_vec2())
                                < get_prototype_level_config().nearby_radius
                            {
                                entity_instance.entity.health.deal_damage(150.0);
                            }
                        }
                        true
                    },
                )),
                card_art_path: "lightning_strike.jpg",
                attack: None,
                health: None,
                description: "Deal 150 damage\nto all units and buildings\nin a small area",
            },
            Card::ReinforcedDoors => CardData {
                name: "Reinforced Doors",
                energy_cost: 2,
                play_fn: PlayFn::WorldPos(SpecificPlayFn::new(
                    |_target,
                     owner,
                     _static_game_state,
                     _semi_static_game_state,
                     dynamic_game_state| {
                        for entity_instance in dynamic_game_state.entities.iter_mut() {
                            if entity_instance.entity.tag == EntityTag::Tower
                                && entity_instance.owner == owner
                            {
                                buff_add_to_entity(
                                    Buff::ExtraHealth(ExtraHealthBuff::new(200.0, Some(f32::MAX))),
                                    &mut entity_instance.entity,
                                );
                            }
                        }
                        true
                    },
                )),
                description: "All your towers\nget +200 health",
                card_art_path: "reinforced_doors.jpg",
                attack: None,
                health: None,
            },
            Card::HigherMotivation => CardData {
                name: "Higher Motivation",
                energy_cost: 3,
                play_fn: PlayFn::WorldPos(SpecificPlayFn::new(
                    |_target,
                     owner,
                     _static_game_state,
                     _semi_static_game_state,
                     dynamic_game_state| {
                        for entity_instance in dynamic_game_state.entities.iter_mut() {
                            if entity_instance.entity.tag == EntityTag::Unit
                                && entity_instance.owner == owner
                            {
                                buff_add_to_entity(
                                    Buff::AttackSpeed(
                                        ArithmeticBuff::new_multiplicative(1.5).with_timeout(10.0),
                                    ),
                                    &mut entity_instance.entity,
                                );
                            }
                        }
                        true
                    },
                )),
                description: "All your units\nget +50% attack speed for 10 seconds",
                card_art_path: "higher_motivation.jpg",
                attack: None,
                health: None,
            },
            Card::SteadyAim => CardData {
                name: "Steady Aim",
                energy_cost: 2,
                play_fn: PlayFn::Entity(
                    SpecificPlayFn::new(
                        |target: EntityTarget,
                         _owner,
                         _static_game_state,
                         _semi_static_game_state,
                         dynamic_game_state| {
                            let Some(entity) =
                                find_entity_mut(&mut dynamic_game_state.entities, Some(target.id))
                            else {
                                return false;
                            };
                            buff_add_to_entity(
                                Buff::AttackDamage(
                                    ArithmeticBuff::new_multiplicative(1.4).with_timeout(f32::MAX),
                                ),
                                &mut entity.entity,
                            );
                            true
                        },
                    )
                    .with_target_is_invalid(
                        |target,
                         _owner,
                         _static_game_state,
                         _semi_static_game_state,
                         dynamic_game_state| {
                            !find_entity(&dynamic_game_state.entities, Some(target.id)).is_some_and(
                                |e| {
                                    e.entity
                                        .attacks
                                        .iter()
                                        .any(|a| a.variant == AttackVariant::RangedAttack)
                                },
                            )
                        },
                    ),
                ),
                description: "Give a ranged unit\n+50% attack damage",
                card_art_path: "steady_aim.jpg",
                attack: None,
                health: None,
            },
            Card::Meteor => CardData {
                name: "Meteor",
                energy_cost: 8,
                play_fn: PlayFn::Entity(
                    SpecificPlayFn::new(
                        |target: EntityTarget,
                         _owner,
                         _static_game_state,
                         _semi_static_game_state,
                         dynamic_game_state| {
                            let Some(entity) =
                                find_entity_mut(&mut dynamic_game_state.entities, Some(target.id))
                            else {
                                return false;
                            };
                            entity.state = EntityState::Dead;
                            true
                        },
                    )
                    .with_target_is_invalid(
                        |target,
                         _owner,
                         _static_game_state,
                         _semi_static_game_state,
                         dynamic_game_state| {
                            !find_entity(&dynamic_game_state.entities, Some(target.id))
                                .is_some_and(|e| e.entity.tag == EntityTag::Tower)
                        },
                    ),
                ),
                description: "Destroy a tower",
                card_art_path: "meteor.jpg",
                attack: None,
                health: None,
            },
        }
    }
}

impl Card {
    pub fn get_texture_path(&self) -> String {
        format!("assets/cards/{}.png", self.get_card_data().name)
    }
}
