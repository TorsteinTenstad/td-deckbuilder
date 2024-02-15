use serde::{Deserialize, Serialize};

use crate::{
    component_attack::{Attack, AttackSpeed},
    component_movement::{Movement, MovementSpeed},
    config::PROJECTILE_RADIUS,
    entity::{AbilityFlag, Entity, EntityState, EntityTag, Health, Spy},
    ids::PlayerId,
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
    Tower,
    SpawnPoint,
    Base,
}

const UNIT_RADIUS: f32 = 36.0;
pub const DEFAULT_UNIT_DETECTION_RADIUS: f32 = 200.0;
const BUILDING_RADIUS: f32 = 64.0;

impl EntityBlueprint {
    pub fn create(&self, owner: PlayerId) -> Entity {
        let tag = match self {
            EntityBlueprint::HomesickWarrior
            | EntityBlueprint::SmallCriminal
            | EntityBlueprint::StreetCriminal
            | EntityBlueprint::Spy
            | EntityBlueprint::RecklessKnight
            | EntityBlueprint::DemonWolf
            | EntityBlueprint::ElfWarrior
            | EntityBlueprint::OldSwordMaster
            | EntityBlueprint::BasicBuilder => EntityTag::Unit,
            EntityBlueprint::Tower | EntityBlueprint::SpawnPoint => EntityTag::Tower,
            EntityBlueprint::Base => EntityTag::Base,
        };
        let state = match tag {
            EntityTag::Unit => EntityState::Moving,
            EntityTag::Tower => EntityState::Attacking,
            EntityTag::Base => EntityState::Passive,
            EntityTag::Bullet => EntityState::Moving,
        };
        let radius = match tag {
            EntityTag::Unit => UNIT_RADIUS,
            EntityTag::Tower => BUILDING_RADIUS,
            EntityTag::Base => BUILDING_RADIUS,
            EntityTag::Bullet => PROJECTILE_RADIUS,
        };
        let mut entity = Entity::new(tag, owner, state);
        entity.radius = radius;
        match self {
            EntityBlueprint::BasicBuilder => {
                entity.health = Health::new(100.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitBuilder;
                entity.attacks.push(Attack {
                    damage: 5.0,
                    ..Attack::default()
                });
            }
            EntityBlueprint::HomesickWarrior => {
                entity.health = Health::new(200.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitHomesickWarrior;
                entity.ability_flags = vec![AbilityFlag::Protector];
                entity.attacks.push(Attack {
                    damage: 20.0,
                    ..Attack::default()
                });
            }
            EntityBlueprint::ElfWarrior => {
                entity.health = Health::new(100.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitElfWarrior;
                entity.attacks.push(Attack {
                    damage: 10.0,
                    attack_speed: AttackSpeed::Fast,
                    ..Attack::default_ranged()
                });
            }
            EntityBlueprint::OldSwordMaster => {
                entity.health = Health::new(200.0);
                entity.movement = Some(Movement::new(MovementSpeed::VerySlow));
                entity.sprite_id = SpriteId::UnitOldSwordMaster;
                entity.attacks.push(Attack {
                    damage: 50.0,
                    ..Attack::default()
                });
            }
            EntityBlueprint::DemonWolf => {
                entity.health = Health::new(200.0);
                entity.movement = Some(Movement::new(MovementSpeed::Fast));
                entity.sprite_id = SpriteId::UnitDemonWolf;
                entity.attacks.push(Attack {
                    damage: 20.0,
                    ..Attack::default()
                });
            }
            EntityBlueprint::SmallCriminal => {
                entity.health = Health::new(200.0);
                entity.movement = Some(Movement::new(MovementSpeed::Fast));
                entity.sprite_id = SpriteId::UnitSmallCriminal;
                entity.attacks.push(Attack {
                    damage: 10.0,
                    ..Attack::default()
                });
            }
            EntityBlueprint::StreetCriminal => {
                entity.health = Health::new(200.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitStreetCriminal;
                entity.attacks.push(Attack {
                    damage: 10.0,
                    attack_speed: AttackSpeed::Fast,
                    ..Attack::default()
                });
            }
            EntityBlueprint::Spy => {
                entity.health = Health::new(200.0);
                entity.movement = Some(Movement::new(MovementSpeed::Default));
                entity.sprite_id = SpriteId::UnitSpy;
                entity.attacks.push(Attack {
                    damage: 20.0,
                    ..Attack::default()
                });
                entity.spy = Some(Spy::new(2));
            }
            EntityBlueprint::RecklessKnight => {
                entity.health = Health::new(100.0);
                entity.movement = Some(Movement::new(MovementSpeed::Fast));
                entity.sprite_id = SpriteId::UnitRecklessKnight;
                entity.attacks.push(Attack {
                    damage: 30.0,
                    ..Attack::default()
                });
            }
            EntityBlueprint::Tower => {
                entity.health = Health::new(500.0);
                entity.sprite_id = SpriteId::BuildingTower;
                entity.attacks.push(Attack {
                    damage: 20.0,
                    ..Attack::default_ranged_tower()
                });
            }
            EntityBlueprint::SpawnPoint => {
                entity.health = Health::new(400.0);
                entity.sprite_id = SpriteId::BuildingSpawnpoint;
                entity.usable_as_spawn_point = true;
            }
            EntityBlueprint::Base => {
                entity.health = Health::new(2000.0);
                entity.sprite_id = SpriteId::BuildingBase;
                entity.usable_as_spawn_point = true;
            }
        }
        entity.hitbox_radius = entity.radius;
        entity
    }
}
