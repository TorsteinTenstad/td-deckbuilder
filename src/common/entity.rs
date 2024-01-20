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

#[derive(Default, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub tag: EntityTag,
    pub owner: PlayerId,
    pub state: EntityState,
    #[serde(skip)]
    pub movement: Option<Movement>,
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub radius: f32,
    pub hitbox_radius: f32,
    pub usable_as_spawn_point: bool,
    pub health: Health,
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
            hitbox_radius: 0.0,
            usable_as_spawn_point: false,
            attacks: Vec::new(),
            seconds_left_to_live: None,
            building_to_construct: None,
        }
    }
}
