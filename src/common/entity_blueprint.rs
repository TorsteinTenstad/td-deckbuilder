use serde::{Deserialize, Serialize};

use crate::{
    component_attack::{Attack, AttackVariant},
    component_movement_behavior::{MovementBehavior, PathMovementBehavior},
    entity::{Entity, EntityState, EntityTag},
    ids::{BuildingLocationId, PlayerId},
    play_target::BuildingSpotTarget,
    textures::SpriteId,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum EntityBlueprint {
    BasicSwordsman,
    BasicRanger,
    BasicTower,
    BasicTowerBuilder,
    SpawnPoint,
    SpawnPointBuilder,
    Base,
}

const UNIT_RADIUS: f32 = 36.0;
const BUILDING_RADIUS: f32 = 64.0;

impl EntityBlueprint {
    pub fn create(&self, owner: PlayerId) -> Entity {
        let tag = match self {
            EntityBlueprint::BasicSwordsman
            | EntityBlueprint::BasicRanger
            | EntityBlueprint::BasicTowerBuilder
            | EntityBlueprint::SpawnPointBuilder => EntityTag::Unit,
            EntityBlueprint::BasicTower | EntityBlueprint::SpawnPoint => EntityTag::Tower,
            EntityBlueprint::Base => EntityTag::Base,
        };
        let state = match self {
            EntityBlueprint::BasicSwordsman
            | EntityBlueprint::BasicRanger
            | EntityBlueprint::BasicTowerBuilder
            | EntityBlueprint::SpawnPointBuilder => EntityState::Moving,
            EntityBlueprint::BasicTower => EntityState::Attacking,
            EntityBlueprint::SpawnPoint | EntityBlueprint::Base => EntityState::Passive,
        };
        let mut entity = Entity::new(tag, owner, state);
        match self {
            EntityBlueprint::BasicTowerBuilder => {
                entity.radius = UNIT_RADIUS;
                entity.health = 100.0;
                entity.hitbox_radius = entity.radius;
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior {
                    path_state: None,
                    speed: 100.0,
                    detection_radius: 150.0,
                });
                entity.sprite_id = SpriteId::UnitSwordsman;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    entity.radius,
                    10.0,
                    0.5,
                    vec![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
                ));
                entity.building_to_construct = Some((
                    BuildingSpotTarget {
                        id: BuildingLocationId(0),
                    },
                    EntityBlueprint::BasicTower,
                ));
            }
            EntityBlueprint::SpawnPointBuilder => {
                entity.radius = UNIT_RADIUS;
                entity.health = 100.0;
                entity.hitbox_radius = entity.radius;
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior {
                    path_state: None,
                    speed: 100.0,
                    detection_radius: 150.0,
                });
                entity.sprite_id = SpriteId::UnitSwordsman;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    entity.radius,
                    10.0,
                    0.5,
                    vec![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
                ));
                entity.building_to_construct = Some((
                    BuildingSpotTarget {
                        id: BuildingLocationId(0),
                    },
                    EntityBlueprint::SpawnPoint,
                ));
            }
            EntityBlueprint::BasicSwordsman => {
                entity.radius = UNIT_RADIUS;
                entity.health = 100.0;
                entity.hitbox_radius = entity.radius;
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior {
                    path_state: None,
                    speed: 100.0,
                    detection_radius: 150.0,
                });
                entity.sprite_id = SpriteId::UnitSwordsman;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    entity.radius,
                    10.0,
                    0.5,
                    vec![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
                ));
            }
            EntityBlueprint::BasicRanger => {
                entity.radius = UNIT_RADIUS;
                entity.health = 50.0;
                entity.hitbox_radius = entity.radius;
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior {
                    path_state: None,
                    speed: 100.0,
                    detection_radius: 150.0,
                });
                entity.sprite_id = SpriteId::UnitArcher;
                entity.attacks.push(Attack::new(
                    AttackVariant::RangedAttack,
                    200.0,
                    10.0,
                    0.5,
                    vec![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
                ));
            }
            EntityBlueprint::BasicTower => {
                let range = 350.0;
                entity.radius = BUILDING_RADIUS;
                entity.health = 200.0;
                entity.hitbox_radius = entity.radius;
                entity.attacks.push(Attack::new(
                    AttackVariant::RangedAttack,
                    range,
                    5.0,
                    0.5,
                    vec![EntityTag::Unit],
                ));
                entity.sprite_id = SpriteId::BuildingTower
            }
            EntityBlueprint::SpawnPoint => {
                entity.radius = BUILDING_RADIUS;
                entity.health = 200.0;
                entity.hitbox_radius = entity.radius;
                entity.usable_as_spawn_point = true;
                entity.sprite_id = SpriteId::BuildingSpawnpoint
            }
            EntityBlueprint::Base => {
                entity.radius = 48.0;
                entity.health = 1000.0;
                entity.hitbox_radius = entity.radius;
                entity.usable_as_spawn_point = true;
                entity.sprite_id = SpriteId::BuildingBase
            }
        }
        entity
    }
}
