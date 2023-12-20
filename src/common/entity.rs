use crate::attack::Attack;
use crate::component_movement_behavior::MovementBehavior;
use crate::entity_blueprint::EntityBlueprint;
use crate::serde_defs::Vec2Def;
use macroquad::math::Vec2;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityTag {
    Base,
    Tower,
    Unit,
    Bullet,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EntityState {
    Moving,
    Attacking,
    Passive,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: u64,
    pub tag: EntityTag,
    pub owner: u64,
    pub state: EntityState,
    pub speed: f32,
    pub movement_behavior: MovementBehavior,
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub radius: f32,
    pub health: f32,
    pub damage_animation: f32,
    pub hitbox_radius: f32,
    pub usable_as_spawn_point: bool,
    pub attacks: Vec<Attack>,
    pub seconds_left_to_live: Option<f32>,
    pub building_to_construct: Option<(u64, EntityBlueprint)>,
}

impl Entity {
    pub fn new(tag: EntityTag, owner: u64, state: EntityState) -> Self {
        Self {
            id: thread_rng().gen(),
            tag,
            owner,
            state,
            speed: 0.0,
            movement_behavior: MovementBehavior::None,
            pos: Vec2::ZERO,
            radius: 0.0,
            health: 0.0,
            damage_animation: 0.0,
            hitbox_radius: 0.0,
            usable_as_spawn_point: false,
            attacks: Vec::new(),
            seconds_left_to_live: None,
            building_to_construct: None,
        }
    }
}
