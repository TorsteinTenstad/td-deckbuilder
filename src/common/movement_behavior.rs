use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

use crate::{get_path_pos, next_path_idx, Direction, Entity, StaticGameState, Vec2Def};

#[derive(Clone, Serialize, Deserialize)]
pub enum MovementBehavior {
    Bullet(BulletMovementBehavior),
    Path(PathMovementBehavior),
    None,
}

impl MovementBehavior {
    pub fn update(
        entity: &mut Entity,
        other_entities: &mut Vec<Entity>,
        dt: f32,
        static_game_state: &StaticGameState,
    ) {
        match &mut entity.movement_behavior {
            MovementBehavior::Path(PathMovementBehavior {
                path_id,
                target_path_idx,
                direction,
                speed,
            }) => {
                if *target_path_idx < static_game_state.paths.get(path_id).unwrap().len() {
                    let target_pos = get_path_pos(static_game_state, *path_id, *target_path_idx);
                    let delta = target_pos - entity.pos;
                    entity.pos += delta.normalize_or_zero() * *speed * dt;
                    let updated_delta = target_pos - entity.pos;
                    if delta.length_squared() < updated_delta.length_squared() {
                        *target_path_idx = next_path_idx(*target_path_idx, *direction)
                    }
                }
            }
            MovementBehavior::Bullet(BulletMovementBehavior {
                velocity,
                target_entity_id,
                speed,
            }) => {
                *velocity = target_entity_id
                    .and_then(|target_entity_id| {
                        other_entities
                            .iter()
                            .find(|entity| entity.id == target_entity_id)
                            .map(|target_entity| {
                                (target_entity.pos - entity.pos).normalize_or_zero() * *speed
                            })
                    })
                    .unwrap_or(*velocity);

                entity.pos += *velocity * dt;
            }
            MovementBehavior::None => {}
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BulletMovementBehavior {
    #[serde(with = "Vec2Def")]
    pub velocity: Vec2,
    pub target_entity_id: Option<u64>,
    pub speed: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathMovementBehavior {
    pub path_id: u64,
    pub target_path_idx: usize,
    pub direction: Direction,
    pub speed: f32,
}
