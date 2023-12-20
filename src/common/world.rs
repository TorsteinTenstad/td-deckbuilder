use crate::{
    entity::{Entity, EntityTag},
    game_state::StaticGameState,
    ids::{EntityId, PathId, PlayerId},
    serde_defs::Vec2Def,
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BuildingLocation {
    #[serde(with = "Vec2Def")]
    pub pos: Vec2,
    pub entity_id: Option<EntityId>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Positive,
    Negative,
}

impl Direction {
    pub fn to_f32(&self) -> f32 {
        match self {
            Direction::Positive => 1.0,
            Direction::Negative => -1.0,
        }
    }
    pub fn to_i32(&self) -> i32 {
        match self {
            Direction::Positive => 1,
            Direction::Negative => -1,
        }
    }
}

pub fn next_path_idx(path_idx: usize, direction: Direction) -> usize {
    let next_path_idx = path_idx as i32 + direction.to_i32();
    if next_path_idx < 0 {
        0
    } else {
        next_path_idx as usize
    }
}
pub fn get_path_pos(static_game_state: &StaticGameState, path_id: PathId, path_idx: usize) -> Vec2 {
    static_game_state
        .paths
        .get(&path_id)
        .unwrap()
        .get(path_idx)
        .map(|(x, y)| Vec2 { x: *x, y: *y })
        .unwrap()
}

pub fn find_entity_in_range<'a>(
    entity_pos: Vec2,
    entity_owner: PlayerId,
    range: f32,
    can_target: &Option<Vec<EntityTag>>,
    other_entities: &'a mut Vec<Entity>,
) -> Option<&'a mut Entity> {
    other_entities
        .iter_mut()
        .filter(|other_entity| other_entity.owner != entity_owner)
        .filter(|other_entity| {
            can_target
                .as_ref()
                .is_some_and(|v| v.contains(&other_entity.tag))
        })
        .filter(|other_entity| {
            (other_entity.pos - entity_pos).length_squared()
                < (range + other_entity.hitbox_radius).powi(2)
        })
        .min_by(|other_entity_a, other_entity_b| {
            let signed_distance_a = (other_entity_a.pos - entity_pos).length_squared()
                - (range + other_entity_a.hitbox_radius).powi(2);
            let signed_distance_b = (other_entity_b.pos - entity_pos).length_squared()
                - (range + other_entity_b.hitbox_radius).powi(2);
            signed_distance_a.partial_cmp(&signed_distance_b).unwrap()
        })
}
