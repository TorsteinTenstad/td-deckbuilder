use crate::buff::ArithmeticBuff;
use crate::component_buff_aura::BuffAura;
use crate::component_health::Health;
use crate::component_movement::Movement;
use crate::component_spy::Spy;
use crate::entity_blueprint::EntityBlueprint;
use crate::ids::EntityId;
use crate::play_target::BuildingSpotTarget;
use crate::serde_defs::Vec2Def;
use crate::{component_attack::Attack, ids::PlayerId, textures::SpriteId};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EntityTag {
    #[default]
    None,
    Base,
    Tower,
    Unit,
    FlyingUnit,
    Bullet,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum EntityState {
    Moving,
    Attacking,
    Passive,
    Building,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AbilityFlag {
    Protector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInstance {
    pub id: EntityId,
    pub owner: PlayerId,
    pub state: EntityState,
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub entity: Entity,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Entity {
    pub tag: EntityTag,
    pub sprite_id: SpriteId,
    pub radius: f32,
    pub hitbox_radius: f32,
    pub ability_flags: Vec<AbilityFlag>,
    pub health: Health,
    pub movement: Option<Movement>,
    pub spy: Option<Spy>,
    pub buff_auras: Vec<BuffAura>,
    pub draw_speed_buff: Option<ArithmeticBuff>,
    pub energy_generation_buff: Option<ArithmeticBuff>,
    pub attacks: Vec<Attack>,
    pub usable_as_spawn_point: bool,
    pub seconds_left_to_live: Option<f32>,
    pub building_to_construct: Option<(BuildingSpotTarget, EntityBlueprint)>,
}

impl Entity {
    pub fn instantiate(self, owner: PlayerId, state: EntityState) -> EntityInstance {
        EntityInstance {
            id: EntityId::new(),
            owner,
            state,
            pos: Vec2::ZERO,
            entity: self,
        }
    }
}
