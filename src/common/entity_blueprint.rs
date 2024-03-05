use serde::{Deserialize, Serialize};

use crate::{
    buff::ArithmeticBuff,
    component_attack::{Attack, AttackSpeed},
    component_health::Health,
    component_movement::{Movement, MovementSpeed},
    component_spy::Spy,
    config::PROJECTILE_RADIUS,
    entity::{AbilityFlag, Entity, EntityTag},
    textures::SpriteId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Dragon,
    WarEagle,
    AirBalloon,
    Tower,
    Farm,
    TradingPlace,
    SpawnPoint,
    Base,
}

const UNIT_RADIUS: f32 = 36.0;
pub const DEFAULT_UNIT_DETECTION_RADIUS: f32 = 200.0;
const BUILDING_RADIUS: f32 = 64.0;

impl EntityBlueprint {
    pub fn create(&self) -> Entity {
        /*
        let state = match tag {
            EntityTag::Unit | EntityTag::FlyingUnit => EntityState::Moving,
            EntityTag::Tower => EntityState::Attacking,
            EntityTag::Base => EntityState::Passive,
            EntityTag::Bullet => EntityState::Moving,
        };
        */
        let mut entity = match self {
            EntityBlueprint::BasicBuilder => Entity {
                tag: EntityTag::Unit,
                health: Health::new(100.0),
                movement: Some(Movement::new(MovementSpeed::Default)),
                sprite_id: SpriteId::UnitBuilder,
                attacks: vec![Attack {
                    damage: 5.0,
                    ..Attack::default()
                }],
                ..Default::default()
            },
            EntityBlueprint::HomesickWarrior => Entity {
                tag: EntityTag::Unit,
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::Default)),
                sprite_id: SpriteId::UnitHomesickWarrior,
                ability_flags: vec![AbilityFlag::Protector],
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default()
                }],
                ..Default::default()
            },
            EntityBlueprint::ElfWarrior => Entity {
                tag: EntityTag::Unit,
                health: Health::new(100.0),
                movement: Some(Movement::new(MovementSpeed::Default)),
                sprite_id: SpriteId::UnitElfWarrior,
                attacks: vec![Attack {
                    damage: 10.0,
                    attack_speed: AttackSpeed::Fast,
                    ..Attack::default_ranged()
                }],
                ..Default::default()
            },
            EntityBlueprint::OldSwordMaster => Entity {
                tag: EntityTag::Unit,
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::VerySlow)),
                sprite_id: SpriteId::UnitOldSwordMaster,
                attacks: vec![Attack {
                    damage: 50.0,
                    ..Attack::default()
                }],
                ..Default::default()
            },
            EntityBlueprint::DemonWolf => Entity {
                tag: EntityTag::Unit,
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::Fast)),
                sprite_id: SpriteId::UnitDemonWolf,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default()
                }],
                ..Default::default()
            },
            EntityBlueprint::SmallCriminal => Entity {
                tag: EntityTag::Unit,
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::Fast)),
                sprite_id: SpriteId::UnitSmallCriminal,
                attacks: vec![Attack {
                    damage: 10.0,
                    ..Attack::default()
                }],
                ..Default::default()
            },
            EntityBlueprint::StreetCriminal => Entity {
                tag: EntityTag::Unit,
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::Default)),
                sprite_id: SpriteId::UnitStreetCriminal,
                attacks: vec![Attack {
                    damage: 10.0,
                    attack_speed: AttackSpeed::Fast,
                    ..Attack::default()
                }],
                ..Default::default()
            },
            EntityBlueprint::Spy => Entity {
                tag: EntityTag::Unit,
                health: Health::new(200.0),
                movement: Some(Movement::new(MovementSpeed::Default)),
                sprite_id: SpriteId::UnitSpy,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default()
                }],
                spy: Some(Spy::new(2)),
                ..Default::default()
            },
            EntityBlueprint::RecklessKnight => Entity {
                tag: EntityTag::Unit,
                health: Health::new(100.0),
                movement: Some(Movement::new(MovementSpeed::Fast)),
                sprite_id: SpriteId::UnitRecklessKnight,
                attacks: vec![Attack {
                    damage: 30.0,
                    ..Attack::default()
                }],
                ..Default::default()
            },
            EntityBlueprint::WarEagle => Entity {
                tag: EntityTag::FlyingUnit,
                health: Health::new(100.0),
                movement: Some(Movement::new(MovementSpeed::Default)),
                sprite_id: SpriteId::UnitWarEagle,
                attacks: vec![Attack {
                    damage: 10.0,
                    ..Attack::default_flying()
                }],
                ..Default::default()
            },
            EntityBlueprint::AirBalloon => Entity {
                tag: EntityTag::FlyingUnit,
                health: Health::new(400.0),
                movement: Some(Movement::new(MovementSpeed::Slow)),
                sprite_id: SpriteId::UnitAirBalloon,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default_flying()
                }],
                ..Default::default()
            },
            EntityBlueprint::Dragon => Entity {
                tag: EntityTag::FlyingUnit,
                health: Health::new(400.0),
                movement: Some(Movement::new(MovementSpeed::Default)),
                sprite_id: SpriteId::UnitDragon,
                attacks: vec![Attack {
                    damage: 40.0,
                    ..Attack::default_flying()
                }],
                ..Default::default()
            },
            EntityBlueprint::Tower => Entity {
                tag: EntityTag::Tower,
                health: Health::new(500.0),
                sprite_id: SpriteId::BuildingTower,
                attacks: vec![Attack {
                    damage: 20.0,
                    ..Attack::default_ranged_tower()
                }],
                ..Default::default()
            },
            EntityBlueprint::Farm => Entity {
                tag: EntityTag::Tower,
                health: Health::new(200.0),
                sprite_id: SpriteId::BuildingFarm,
                draw_speed_buff: Some(ArithmeticBuff {
                    multiplier: 1.4,
                    ..Default::default()
                }),
                ..Default::default()
            },
            EntityBlueprint::TradingPlace => Entity {
                tag: EntityTag::Tower,
                health: Health::new(200.0),
                sprite_id: SpriteId::BuildingTradingPlace,
                energy_generation_buff: Some(ArithmeticBuff {
                    multiplier: 1.4,
                    ..Default::default()
                }),
                ..Default::default()
            },
            EntityBlueprint::SpawnPoint => Entity {
                tag: EntityTag::Tower,
                health: Health::new(400.0),
                sprite_id: SpriteId::BuildingHut,
                usable_as_spawn_point: true,
                ..Default::default()
            },
            EntityBlueprint::Base => Entity {
                tag: EntityTag::Base,
                health: Health::new(2000.0),
                sprite_id: SpriteId::BuildingBase,
                usable_as_spawn_point: true,
                ..Default::default()
            },
        };
        let radius = match entity.tag {
            EntityTag::None => {
                debug_assert!(false);
                0.0
            }
            EntityTag::Unit | EntityTag::FlyingUnit => UNIT_RADIUS,
            EntityTag::Tower => BUILDING_RADIUS,
            EntityTag::Base => BUILDING_RADIUS,
            EntityTag::Bullet => PROJECTILE_RADIUS,
        };
        entity.hitbox_radius = radius;
        entity.radius = radius;
        entity
    }
}
