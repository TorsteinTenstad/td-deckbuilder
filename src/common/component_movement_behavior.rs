use crate::{
    entity::Entity,
    game_state::StaticGameState,
    ids::{EntityId, PathId},
    play_target::UnitSpawnpointTarget,
    serde_defs::Vec2Def,
    world::{find_entity, get_path_pos, next_path_idx, Direction},
};
use macroquad::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum MovementBehavior {
    Bullet(BulletMovementBehavior),
    Path(PathMovementBehavior),
    None,
}

impl MovementBehavior {
    pub fn update(
        entity: &mut Entity,
        entities: &mut Vec<Entity>,
        dt: f32,
        static_game_state: &StaticGameState,
    ) {
        match &mut entity.movement_behavior {
            MovementBehavior::Path(PathMovementBehavior {
                path_id,
                target_path_idx,
                direction,
            }) => {
                if *target_path_idx < static_game_state.paths.get(path_id).unwrap().len() {
                    let target_pos = get_path_pos(static_game_state, *path_id, *target_path_idx);
                    let delta = target_pos - entity.pos;
                    entity.pos += delta.normalize_or_zero() * entity.speed * dt;
                    let updated_delta = target_pos - entity.pos;
                    if delta.length_squared() < updated_delta.length_squared() {
                        *target_path_idx = next_path_idx(*target_path_idx, *direction)
                    }
                }
            }
            MovementBehavior::Bullet(BulletMovementBehavior {
                velocity,
                target_entity_id,
            }) => {
                *velocity = find_entity(entities, *target_entity_id)
                    .map(|target_entity| {
                        (target_entity.pos - entity.pos).normalize_or_zero() * entity.speed
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
    pub target_entity_id: Option<EntityId>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathMovementBehavior {
    pub path_id: PathId,
    pub target_path_idx: usize,
    pub direction: Direction,
}

impl From<UnitSpawnpointTarget> for PathMovementBehavior {
    fn from(target: UnitSpawnpointTarget) -> Self {
        Self {
            path_id: target.path_id,
            target_path_idx: next_path_idx(target.path_idx, target.direction),
            direction: target.direction,
        }
    }
}
