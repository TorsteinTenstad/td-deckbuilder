use serde::{Deserialize, Serialize};

use crate::{
    component_attack::{Attack, AttackRange, AttackSpeed, AttackVariant},
    component_movement_behavior::{MovementBehavior, MovementSpeed, PathMovementBehavior},
    entity::{Entity, EntityState, EntityTag, Health},
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
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior::new(
                    MovementSpeed::Default,
                    DEFAULT_UNIT_DETECTION_RADIUS,
                ));
                entity.sprite_id = SpriteId::UnitBuilder;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    AttackRange::Melee,
                    10.0,
                    AttackSpeed::Default,
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
                entity.health = Health::new(100.0);
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior::new(
                    MovementSpeed::Default,
                    DEFAULT_UNIT_DETECTION_RADIUS,
                ));
                entity.sprite_id = SpriteId::UnitBuilder;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    AttackRange::Melee,
                    10.0,
                    AttackSpeed::Default,
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
                entity.health = Health::new(100.0);
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior::new(
                    MovementSpeed::Default,
                    DEFAULT_UNIT_DETECTION_RADIUS,
                ));
                entity.sprite_id = SpriteId::UnitSwordsman;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    AttackRange::Melee,
                    10.0,
                    AttackSpeed::Default,
                    vec![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
                ));
            }
            EntityBlueprint::Priest => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(100.0);
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior::new(
                    MovementSpeed::Default,
                    DEFAULT_UNIT_DETECTION_RADIUS,
                ));
                entity.sprite_id = SpriteId::UnitPriest;
                entity.attacks.push(Attack::new(
                    AttackVariant::Heal,
                    AttackRange::Default,
                    10.0,
                    AttackSpeed::Default,
                    vec![EntityTag::Unit],
                ));
            }
            EntityBlueprint::DemonPig => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(50.0);
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior::new(
                    MovementSpeed::Default,
                    DEFAULT_UNIT_DETECTION_RADIUS,
                ));
                entity.sprite_id = SpriteId::UnitDemonPig;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    AttackRange::Melee,
                    3.0,
                    AttackSpeed::Fast,
                    vec![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
                ));
            }
            EntityBlueprint::BasicRanger => {
                entity.radius = UNIT_RADIUS;
                entity.health = Health::new(50.0);
                entity.movement_behavior = MovementBehavior::Path(PathMovementBehavior::new(
                    MovementSpeed::Default,
                    DEFAULT_UNIT_DETECTION_RADIUS,
                ));
                entity.sprite_id = SpriteId::UnitArcher;
                entity.attacks.push(Attack::new(
                    AttackVariant::RangedAttack,
                    AttackRange::Default,
                    10.0,
                    AttackSpeed::Default,
                    vec![EntityTag::Base, EntityTag::Tower, EntityTag::Unit],
                ));
            }
            EntityBlueprint::BasicTower => {
                entity.radius = BUILDING_RADIUS;
                entity.health = Health::new(200.0);
                entity.attacks.push(Attack::new(
                    AttackVariant::RangedAttack,
                    AttackRange::Default,
                    5.0,
                    AttackSpeed::Default,
                    vec![EntityTag::Unit],
                ));
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
