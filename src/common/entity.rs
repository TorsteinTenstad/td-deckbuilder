use crate::component_movement_behavior::MovementBehavior;
use crate::entity_blueprint::EntityBlueprint;
use crate::ids::EntityId;
use crate::outgoing_buff::{OutgoingBuff, OutgoingBuffInstance};
use crate::play_target::{BuildingSpotTarget, EntityTarget, WorldPosTarget};
use crate::serde_defs::Vec2Def;
use crate::{component_attack::Attack, ids::PlayerId, textures::SpriteId};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityTag {
    Base,
    Tower,
    Unit,
    Bullet,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum EntityState {
    Moving,
    Attacking,
    Passive,
    Building,
    Dead,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MoveTarget {
    WorldPos(WorldPosTarget),
    BuildingSpot(BuildingSpotTarget),
    Entity(EntityTarget),
    None,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub tag: EntityTag,
    pub owner: PlayerId,
    pub state: EntityState,
    pub movement_behavior: MovementBehavior,
    pub move_target: MoveTarget,
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub radius: f32,
    pub max_health: f32,
    pub health: f32,
    pub damage_animation: f32,
    pub hitbox_radius: f32,
    pub usable_as_spawn_point: bool,
    pub buffs: Vec<OutgoingBuffInstance>,
    pub attacks: Vec<Attack>,
    pub seconds_left_to_live: Option<f32>,
    pub building_to_construct: Option<(BuildingSpotTarget, EntityBlueprint)>,
    pub sprite_id: SpriteId,
}

impl Entity {
    pub fn new(tag: EntityTag, owner: PlayerId, state: EntityState) -> Self {
        Self {
            id: EntityId::new(),
            tag,
            owner,
            state,
            sprite_id: SpriteId::Empty,
            movement_behavior: MovementBehavior::None,
            move_target: MoveTarget::None,
            pos: Vec2::ZERO,
            radius: 0.0,
            max_health: 0.0,
            health: 0.0,
            damage_animation: 0.0,
            hitbox_radius: 0.0,
            usable_as_spawn_point: false,
            attacks: Vec::new(),
            buffs: Vec::new(),
            seconds_left_to_live: None,
            building_to_construct: None,
        }
    }

    pub fn get_buffs(&self) -> impl Iterator<Item = &OutgoingBuff> {
        self.buffs.iter().map(|buff_instance| &buff_instance.buff)
    }
}
