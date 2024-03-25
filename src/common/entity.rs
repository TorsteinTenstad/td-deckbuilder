use crate::buff::ArithmeticBuff;
use crate::component_buff_aura::BuffAura;
use crate::component_health::Health;
use crate::component_movement::{Movement, MovementSpeed};
use crate::component_self_buff::SelfBuff;
use crate::component_spy::Spy;
use crate::config;
use crate::entity_blueprint::EntityBlueprint;
use crate::ids::EntityId;
use crate::play_target::BuildingLocationTarget;
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

impl From<EntityTag> for usize {
    fn from(val: EntityTag) -> Self {
        val as usize
    }
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
    pub self_buffs: Vec<SelfBuff>,
    pub buff_auras: Vec<BuffAura>,
    pub draw_speed_buff: Option<ArithmeticBuff>,
    pub energy_generation_buff: Option<ArithmeticBuff>,
    pub attacks: Vec<Attack>,
    pub usable_as_spawn_point: bool,
    pub seconds_left_to_live: Option<f32>,
    pub building_to_construct: Option<(BuildingLocationTarget, EntityBlueprint)>,
}

impl Entity {
    pub fn instantiate(self, owner: PlayerId, pos: Vec2) -> EntityInstance {
        EntityInstance {
            id: EntityId::new(),
            owner,
            state: EntityState::Moving,
            pos,
            entity: self,
        }
    }

    pub fn default_unit() -> Self {
        Self {
            tag: EntityTag::Unit,
            radius: config::UNIT_RADIUS,
            hitbox_radius: config::UNIT_RADIUS,
            movement: Some(Movement::new(MovementSpeed::Default)),
            ..Default::default()
        }
    }

    pub fn default_flying_unit() -> Self {
        Self {
            tag: EntityTag::FlyingUnit,
            radius: config::UNIT_RADIUS,
            hitbox_radius: config::UNIT_RADIUS,
            movement: Some(Movement::new(MovementSpeed::Default)),
            ..Default::default()
        }
    }

    pub fn default_tower() -> Self {
        Self {
            tag: EntityTag::Tower,
            radius: config::BUILDING_RADIUS,
            hitbox_radius: config::BUILDING_RADIUS,
            ..Default::default()
        }
    }

    pub fn default_base() -> Self {
        Self {
            tag: EntityTag::Base,
            radius: config::BUILDING_RADIUS,
            hitbox_radius: config::BUILDING_RADIUS,
            ..Default::default()
        }
    }
}
