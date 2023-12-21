use serde::{Deserialize, Serialize};

use crate::{
    component_attack::{Attack, AttackVariant},
    entity::{Entity, EntityState, EntityTag},
    ids::PlayerId,
    textures::SpriteId,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum EntityBlueprint {
    BasicSwordsman,
    BasicRanger,
    BasicTower,
    SpawnPointTest,
}

const UNIT_RADIUS: f32 = 36.0;
const BUILDING_RADIUS: f32 = 24.0;

impl EntityBlueprint {
    pub fn create(&self, owner: PlayerId) -> Entity {
        let tag = match self {
            EntityBlueprint::BasicSwordsman | EntityBlueprint::BasicRanger => EntityTag::Unit,
            EntityBlueprint::BasicTower | EntityBlueprint::SpawnPointTest => EntityTag::Tower,
        };
        let state = match self {
            EntityBlueprint::BasicSwordsman | EntityBlueprint::BasicRanger => EntityState::Moving,
            EntityBlueprint::BasicTower => EntityState::Attacking,
            EntityBlueprint::SpawnPointTest => EntityState::Passive,
        };
        let mut entity = Entity::new(tag, owner, state);
        match self {
            EntityBlueprint::BasicSwordsman => {
                entity.radius = UNIT_RADIUS;
                entity.health = 100.0;
                entity.hitbox_radius = entity.radius;
                entity.speed = 100.0;
                entity.sprite_id = SpriteId::UnitSwordsman;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    entity.radius,
                    10.0,
                    0.5,
                ));
            }
            EntityBlueprint::BasicRanger => {
                entity.radius = UNIT_RADIUS;
                entity.health = 50.0;
                entity.hitbox_radius = entity.radius;
                entity.speed = 100.0;
                entity.sprite_id = SpriteId::UnitArcher;
                entity
                    .attacks
                    .push(Attack::new(AttackVariant::RangedAttack, 200.0, 10.0, 0.5));
            }
            EntityBlueprint::BasicTower => {
                let range = 350.0;
                entity.radius = BUILDING_RADIUS;
                entity.health = 200.0;
                entity.hitbox_radius = range / 2.0;
                entity
                    .attacks
                    .push(Attack::new(AttackVariant::RangedAttack, range, 5.0, 0.5));
            }
            EntityBlueprint::SpawnPointTest => {
                entity.radius = BUILDING_RADIUS;
                entity.health = 200.0;
                entity.hitbox_radius = 250.0;
                entity.usable_as_spawn_point = true;
            }
        }
        entity
    }
}
