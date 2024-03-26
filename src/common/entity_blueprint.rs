use crate::{
    buff::{ArithmeticBuff, Buff},
    component_attack::{Attack, AttackInterval, TargetPool},
    component_buff_source::{BuffCondition, BuffRange, BuffSource, BuffTargetFilter},
    component_health::Health,
    component_movement::{Movement, MovementSpeed},
    component_spy::Spy,
    entity::{AbilityFlag, Entity, EntityTag},
    entity_filter::EntityFilter,
    enum_flags::{flags, EnumFlags},
    textures::SpriteId,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, EnumIter)]
pub enum EntityBlueprint {
    BasicBuilder,
    HomesickWarrior,
    ElfWarrior,
    OldSwordMaster,
    DemonWolf,
    SmallCriminal,
    StreetCriminal,
    Spy,
    RecklessKnight,
    Governor,
    Dragon,
    WarEagle,
    AirBalloon,
    Tower,
    SmallTower,
    Watchtower,
    Wall,
    Farm,
    TradingPlace,
    SpawnPoint,
    Base,
}

impl EntityBlueprint {
    pub fn iter() -> impl Iterator<Item = EntityBlueprint> {
        <EntityBlueprint as IntoEnumIterator>::iter()
    }
}

impl EntityBlueprint {
    pub fn get_health(&self) -> Option<i32> {
        Some(self.create().health.max_health as i32)
    }
    pub fn get_attack(&self) -> Option<i32> {
        let entity = self.create();
        if entity.attacks.len() > 1 {
            return None;
        }
        entity.attacks.first().map(|attack| attack.damage as i32)
    }
    pub fn create(&self) -> Entity {
        match self {
            EntityBlueprint::BasicBuilder => Entity {
                health: Health::new(100.0),
                sprite_id: SpriteId::UnitBuilder,
                attacks: vec![Attack {
                    damage: 5.0,
                    ..Attack::default()
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::HomesickWarrior => Entity {
                health: Health::new(200.0),
                sprite_id: SpriteId::UnitHomesickWarrior,
                ability_flags: vec![AbilityFlag::Protector],
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default()
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::ElfWarrior => Entity {
                health: Health::new(100.0),
                sprite_id: SpriteId::UnitElfWarrior,
                attacks: vec![Attack {
                    damage: 10.0,
                    attack_interval: AttackInterval::Fast,
                    ..Attack::default_ranged()
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::OldSwordMaster => Entity {
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::VerySlow)),
                sprite_id: SpriteId::UnitOldSwordMaster,
                attacks: vec![Attack {
                    damage: 50.0,
                    ..Attack::default()
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::DemonWolf => Entity {
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::Fast)),
                sprite_id: SpriteId::UnitDemonWolf,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default()
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::SmallCriminal => Entity {
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::Fast)),
                sprite_id: SpriteId::UnitSmallCriminal,
                attacks: vec![Attack {
                    damage: 10.0,
                    ..Attack::default()
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::StreetCriminal => Entity {
                health: Health::new(200.0),
                sprite_id: SpriteId::UnitStreetCriminal,
                attacks: vec![Attack {
                    damage: 10.0,
                    attack_interval: AttackInterval::Fast,
                    ..Attack::default()
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::Spy => Entity {
                health: Health::new(200.0),
                sprite_id: SpriteId::UnitSpy,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default()
                }],
                spy: Some(Spy::new(2)),
                ..Entity::default_unit()
            },
            EntityBlueprint::RecklessKnight => Entity {
                health: Health::new(100.0),
                movement: Some(Movement::new(MovementSpeed::Fast)),
                sprite_id: SpriteId::UnitRecklessKnight,
                attacks: vec![Attack {
                    damage: 30.0,
                    ..Attack::default()
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::Governor => Entity {
                health: Health::new(300.0),
                sprite_id: SpriteId::UnitGovernor,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default()
                }],
                buff_sources: vec![BuffSource {
                    buff: Buff::AttackDamage(ArithmeticBuff::new_additive(5.0)),
                    condition: BuffCondition::EntityFilter(EntityFilter {
                        range_filter: None,
                        pool_filter: Some(TargetPool::Allies),
                        tag_filter: Some(flags![EntityTag::Tower]),
                    }),
                    target_filter: BuffTargetFilter::Me,
                }],
                ..Entity::default_unit()
            },
            EntityBlueprint::WarEagle => Entity {
                health: Health::new(100.0),
                sprite_id: SpriteId::UnitWarEagle,
                attacks: vec![Attack {
                    damage: 10.0,
                    ..Attack::default_flying()
                }],
                ..Entity::default_flying_unit()
            },
            EntityBlueprint::AirBalloon => Entity {
                health: Health::new(400.0),
                movement: Some(Movement::new(MovementSpeed::Slow)),
                sprite_id: SpriteId::UnitAirBalloon,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default_flying()
                }],
                ..Entity::default_flying_unit()
            },
            EntityBlueprint::Dragon => Entity {
                health: Health::new(400.0),
                sprite_id: SpriteId::UnitDragon,
                attacks: vec![Attack {
                    damage: 40.0,
                    ..Attack::default_flying()
                }],
                ..Entity::default_flying_unit()
            },
            EntityBlueprint::Tower => Entity {
                health: Health::new(500.0),
                sprite_id: SpriteId::BuildingTower,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default_ranged_tower()
                }],
                ..Entity::default_tower()
            },
            EntityBlueprint::SmallTower => Entity {
                health: Health::new(300.0),
                sprite_id: SpriteId::BuildingTower,
                buff_sources: vec![BuffSource {
                    buff: Buff::AttackSpeed(ArithmeticBuff::new_multiplicative(1.2)),
                    condition: BuffCondition::EntityFilter(EntityFilter {
                        range_filter: Some(BuffRange::Default),
                        pool_filter: Some(TargetPool::Allies),
                        tag_filter: Some(flags![EntityTag::Tower]),
                    }),
                    target_filter: BuffTargetFilter::Me,
                }],
                attacks: vec![Attack {
                    damage: 10.0,
                    ..Attack::default_ranged_tower()
                }],
                ..Entity::default_tower()
            },
            EntityBlueprint::Watchtower => Entity {
                health: Health::new(500.0),
                sprite_id: SpriteId::BuildingTower,
                attacks: vec![Attack {
                    damage: 10.0,
                    ..Attack::default_ranged_tower()
                }],
                buff_sources: vec![BuffSource {
                    buff: Buff::AttackRange(ArithmeticBuff::new_multiplicative(2.0)),
                    condition: BuffCondition::AlwaysSingle,
                    target_filter: BuffTargetFilter::EntityFilter(EntityFilter {
                        range_filter: Some(BuffRange::Default),
                        pool_filter: Some(TargetPool::Allies),
                        tag_filter: None,
                    }),
                }],
                ..Entity::default_tower()
            },

            EntityBlueprint::Wall => Entity {
                health: Health::new(1000.0),
                sprite_id: SpriteId::BuildingTower,
                ..Entity::default_tower()
            },
            EntityBlueprint::Farm => Entity {
                health: Health::new(200.0),
                sprite_id: SpriteId::BuildingFarm,
                draw_speed_buff: Some(ArithmeticBuff {
                    multiplier: 1.4,
                    ..Default::default()
                }),
                ..Entity::default_tower()
            },
            EntityBlueprint::TradingPlace => Entity {
                health: Health::new(200.0),
                sprite_id: SpriteId::BuildingTradingPlace,
                energy_generation_buff: Some(ArithmeticBuff {
                    multiplier: 1.4,
                    ..Default::default()
                }),
                ..Entity::default_tower()
            },
            EntityBlueprint::SpawnPoint => Entity {
                health: Health::new(400.0),
                sprite_id: SpriteId::BuildingHut,
                usable_as_spawn_point: true,
                ..Entity::default_tower()
            },
            EntityBlueprint::Base => Entity {
                health: Health::new(2000.0),
                sprite_id: SpriteId::BuildingBase,
                usable_as_spawn_point: true,
                ..Entity::default_base()
            },
        }
    }
}
