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

#[derive(Clone, Serialize, Deserialize)]
pub struct BulletMovementBehavior {
    #[serde(with = "Vec2Def")]
    pub velocity: Vec2,
    pub target_entity_id: Option<EntityId>,
    pub speed: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathMovementBehavior {
    pub speed: f32,
    pub detection_radius: f32,
    pub path_state: Option<PathState>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathState {
    pub path_id: PathId,
    pub target_path_idx: usize,
    pub direction: Direction,
}

impl PathState {
    pub fn incr(&mut self) {
        self.target_path_idx = next_path_idx(self.target_path_idx, self.direction);
    }
}

impl MovementBehavior {
    pub fn update(
        entity: &mut Entity,
        entities: &mut Vec<Entity>,
        dt: f32,
        static_game_state: &StaticGameState,
    ) {
        match &mut entity.movement_behavior {
            MovementBehavior::Path(path_movement_behavior) => {
                let path_state = path_movement_behavior.path_state.as_mut().unwrap();
                if path_state.target_path_idx
                    < static_game_state
                        .paths
                        .get(&path_state.path_id)
                        .unwrap()
                        .len()
                {
                    let target_pos = get_path_pos(
                        static_game_state,
                        path_state.path_id,
                        path_state.target_path_idx,
                    );
                    let delta = target_pos - entity.pos;
                    entity.pos += delta.normalize_or_zero() * path_movement_behavior.speed * dt;
                    let updated_delta = target_pos - entity.pos;
                    if delta.length_squared() < updated_delta.length_squared() {
                        path_state.incr();
                    }
                }
            }
            MovementBehavior::Bullet(BulletMovementBehavior {
                speed,
                velocity,
                target_entity_id,
            }) => {
                *velocity = find_entity(entities, *target_entity_id)
                    .map(|target_entity| {
                        (target_entity.pos - entity.pos).normalize_or_zero() * *speed
                    })
                    .unwrap_or(*velocity);

                entity.pos += *velocity * dt;
            }
            MovementBehavior::None => {}
        }
    }
}

impl From<UnitSpawnpointTarget> for PathState {
    fn from(target: UnitSpawnpointTarget) -> Self {
        Self {
            path_id: target.path_id,
            target_path_idx: next_path_idx(target.path_idx, target.direction),
            direction: target.direction,
        }
    }
}
