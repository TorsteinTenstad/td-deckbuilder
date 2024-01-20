use serde::{Deserialize, Serialize};

use crate::{
    component_attack::{Attack, AttackSpeed, TargetPool},
    component_movement::{Movement, MovementSpeed},
    entity::{AbilityFlag, Entity, EntityState, EntityTag, Health},
    ids::{BuildingLocationId, PlayerId},
    play_target::BuildingSpotTarget,
    textures::SpriteId,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum EntityBlueprint {
    BasicSwordsman,
    Priest,
    DemonPig,
    BasicRanger,
    BasicTower,
    BasicTowerBuilder,
    SpawnPoint,
    SpawnPointBuilder,
    Base,
}

const UNIT_RADIUS: f32 = 36.0;
pub const DEFAULT_UNIT_DETECTION_RADIUS: f32 = 200.0;
const BUILDING_RADIUS: f32 = 64.0;

impl EntityBlueprint {
    pub fn create(&self, owner: PlayerId) -> Entity {
        let tag = match self {
            EntityBlueprint::BasicSwordsman
            | EntityBlueprint::DemonPig
            | EntityBlueprint::BasicRanger
            | EntityBlueprint::BasicTowerBuilder
            | EntityBlueprint::Priest
            | EntityBlueprint::SpawnPointBuilder => EntityTag::Unit,
            EntityBlueprint::BasicTower | EntityBlueprint::SpawnPoint => EntityTag::Tower,
            EntityBlueprint::Base => EntityTag::Base,
        };
        let state = match self {
            EntityBlueprint::BasicSwordsman
            | EntityBlueprint::DemonPig
            | EntityBlueprint::BasicRanger
            | EntityBlueprint::BasicTowerBuilder
            | EntityBlueprint::Priest
            | EntityBlueprint::SpawnPointBuilder => EntityState::Moving,
            EntityBlueprint::BasicTower => EntityState::Attacking,
            EntityBlueprint::SpawnPoint | EntityBlueprint::Base => EntityState::Passive,
        };
        let mut entity = Entity::new(tag, owner, state);
        match self {
            EntityBlueprint::BasicTowerBuilder => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(100.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitBuilder;
                entity.attacks.push(Attack {
                    damage: 10.0,
                    ..Attack::default()
                });
                entity.building_to_construct = Some((
                    BuildingSpotTarget {
                        id: BuildingLocationId(0),
                    },
                    EntityBlueprint::BasicTower,
                ));
            }
            EntityBlueprint::SpawnPointBuilder => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(100.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitBuilder;
                entity.attacks.push(Attack {
                    damage: 10.0,
                    ..Attack::default()
                });
                entity.building_to_construct = Some((
                    BuildingSpotTarget {
                        id: BuildingLocationId(0),
                    },
                    EntityBlueprint::SpawnPoint,
                ));
            }
            EntityBlueprint::BasicSwordsman => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(100.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitSwordsman;
                entity.ability_flags.push(AbilityFlag::Protector);
                entity.attacks.push(Attack {
                    damage: 10.0,
                    ..Attack::default()
                });
            }
            EntityBlueprint::Priest => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(100.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitPriest;

                entity.attacks.push(Attack {
                    damage: -10.0,
                    target_pool: TargetPool::Allies,
                    can_target: vec![EntityTag::Unit],
                    ..Attack::default()
                });
            }
            EntityBlueprint::DemonPig => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(50.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitDemonPig;

                entity.attacks.push(Attack {
                    damage: 3.0,
                    attack_speed: AttackSpeed::Fast,
                    ..Attack::default()
                });
            }
            EntityBlueprint::BasicRanger => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(50.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitArcher;

                entity.attacks.push(Attack {
                    damage: 10.0,
                    ..Attack::default_ranged()
                });
            }
            EntityBlueprint::BasicTower => {
                entity.radius = BUILDING_RADIUS;
                entity.health = Health::new(200.0);

                entity.attacks.push(Attack {
                    damage: 5.0,
                    can_target: vec![EntityTag::Unit],
                    ..Attack::default_ranged()
                });
                entity.sprite_id = SpriteId::BuildingTower
            }
            EntityBlueprint::SpawnPoint => {
                entity.radius = BUILDING_RADIUS;
                entity.health = Health::new(200.0);
                entity.usable_as_spawn_point = true;
                entity.sprite_id = SpriteId::BuildingSpawnpoint
            }
            EntityBlueprint::Base => {
                entity.radius = 48.0;
                entity.health = Health::new(1000.0);
                entity.usable_as_spawn_point = true;
                entity.sprite_id = SpriteId::BuildingBase
            }
        }
        entity.hitbox_radius = entity.radius;
        entity
    }
}
