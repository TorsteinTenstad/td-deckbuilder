use std::collections::HashSet;

use crate::buff::ExtraHealthBuff;
use crate::component_movement::Movement;
use crate::entity_blueprint::EntityBlueprint;
use crate::ids::EntityId;
use crate::play_target::BuildingSpotTarget;
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

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Health {
    pub max_health: f32,
    pub health: f32,
    pub extra_health_buffs: Vec<ExtraHealthBuff>,
    pub damage_animation: f32,
}
impl Health {
    pub fn new(max_health: f32) -> Self {
        Self {
            max_health,
            health: max_health,
            ..Default::default()
        }
    }

    pub fn deal_damage(&mut self, damage: f32) {
        let mut damage = damage;
        for buff in self.extra_health_buffs.iter_mut() {
            if buff.health <= 0.0 {
                break;
            }
            let damage_to_take = damage.min(buff.health);
            damage -= damage_to_take;
            buff.health -= damage_to_take;
        }
        self.health -= damage as f32;
        self.damage_animation = 0.1;
    }

    pub fn heal(&mut self, damage: f32) {
        self.health += damage as f32;
        self.health = self.health.min(self.max_health);
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum AbilityFlag {
    Protector,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Spy {
    pub hide_capacity: u32,
    pub is_hidden_from: HashSet<EntityId>,
}
impl Spy {
    pub fn new(hide_capacity: u32) -> Self {
        Self {
            hide_capacity,
            is_hidden_from: HashSet::new(),
        }
    }
    pub fn is_hidden(&self) -> bool {
        self.is_hidden_from.len() <= self.hide_capacity as usize
    }
    pub fn can_hide_from(&mut self, entity_id: EntityId, entity_tag: EntityTag) -> bool {
        if entity_tag == EntityTag::Unit {
            self.is_hidden_from.insert(entity_id);
        }
        self.is_hidden()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub tag: EntityTag,
    pub owner: PlayerId,
    pub state: EntityState,
    pub movement: Option<Movement>,
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub radius: f32,
    pub hitbox_radius: f32,
    pub ability_flags: Vec<AbilityFlag>,
    pub usable_as_spawn_point: bool,
    pub health: Health,
    pub spy: Option<Spy>,
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
            movement: None,
            pos: Vec2::ZERO,
            radius: 0.0,
            health: Health::default(),
            spy: None,
            hitbox_radius: 0.0,
            ability_flags: Vec::new(),
            usable_as_spawn_point: false,
            attacks: Vec::new(),
            seconds_left_to_live: None,
            building_to_construct: None,
        }
    }
}
