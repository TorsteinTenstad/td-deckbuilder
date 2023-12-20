use serde::{Deserialize, Serialize};

use crate::{
    component_attack::{Attack, AttackVariant},
    entity::{Entity, EntityState, EntityTag},
    ids::PlayerId,
    textures::SpriteId,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum EntityBlueprint {
    BasicUnit,
    BasicRanger,
    BasicTower,
    SpawnPointTest,
}

// TODO: state: EntityState may be defined by Blueprint
impl EntityBlueprint {
    pub fn create(&self, owner: PlayerId) -> Entity {
        let tag = match self {
            EntityBlueprint::BasicUnit | EntityBlueprint::BasicRanger => EntityTag::Unit,
            EntityBlueprint::BasicTower | EntityBlueprint::SpawnPointTest => EntityTag::Tower,
        };
        let state = match self {
            EntityBlueprint::BasicUnit | EntityBlueprint::BasicRanger => EntityState::Moving,
            EntityBlueprint::BasicTower => EntityState::Attacking,
            EntityBlueprint::SpawnPointTest => EntityState::Passive,
        };
        let mut entity = Entity::new(tag, owner, state);
        match self {
            EntityBlueprint::BasicUnit => {
                entity.radius = 24.0;
                entity.health = 100.0;
                entity.hitbox_radius = entity.radius;
                entity.speed = 100.0;
                entity.sprite_id = SpriteId::UnitSoldier;
                entity.attacks.push(Attack::new(
                    AttackVariant::MeleeAttack,
                    entity.radius,
                    10.0,
                    0.5,
                ));
            }
            EntityBlueprint::BasicRanger => {
                entity.radius = 24.0;
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
                entity.radius = 24.0;
                entity.health = 200.0;
                entity.hitbox_radius = range / 2.0;
                entity
                    .attacks
                    .push(Attack::new(AttackVariant::RangedAttack, range, 5.0, 0.5));
            }
            EntityBlueprint::SpawnPointTest => {
                entity.radius = 24.0;
                entity.health = 200.0;
                entity.hitbox_radius = 250.0;
                entity.usable_as_spawn_point = true;
            }
        }
        entity
    }
}
