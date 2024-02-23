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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityTag {
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
pub struct Entity {
    pub id: EntityId,
    pub tag: EntityTag,
    pub owner: PlayerId,
    pub state: EntityState,
    pub sprite_id: SpriteId,
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub radius: f32,
    pub hitbox_radius: f32,
    pub ability_flags: Vec<AbilityFlag>,
    pub health: Health,
    pub movement: Option<Movement>,
    pub spy: Option<Spy>,
    pub buff_auras: Vec<BuffAura>,
    pub attacks: Vec<Attack>,
    pub usable_as_spawn_point: bool,
    pub seconds_left_to_live: Option<f32>,
    pub building_to_construct: Option<(BuildingSpotTarget, EntityBlueprint)>,
}

impl Entity {
    pub fn new(tag: EntityTag, owner: PlayerId, state: EntityState) -> Self {
        Self {
            id: EntityId::new(),
            tag,
            owner,
            state,
            sprite_id: SpriteId::Empty,
            pos: Vec2::ZERO,
            radius: 0.0,
            hitbox_radius: 0.0,
            ability_flags: Vec::new(),
            health: Health::default(),
            movement: None,
            spy: None,
            buff_auras: Vec::new(),
            attacks: Vec::new(),
            usable_as_spawn_point: false,
            seconds_left_to_live: None,
            building_to_construct: None,
        }
    }
}
