use std::collections::HashMap;

use macroquad::math::Vec2;

use crate::{play_target::UnitSpawnpointTarget, DynamicGameState, Entity, StaticGameState};

fn get_closest_path_point(
    paths: &HashMap<u64, Vec<(f32, f32)>>,
    entity: &Entity,
) -> Option<(u64, f32)> {
    paths
        .iter()
        .filter_map(|(path_id, path)| {
            path.iter()
                .enumerate()
                .map(|(path_pos, (x, y))| {
                    (
                        *path_id,
                        path_pos as f32 + (entity.pos - Vec2::new(*x, *y)).length(),
                    )
                })
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(path_id, path_pos)| {
            (
                path_id,
                path_pos / paths.get(&path_id).unwrap().len() as f32,
            )
        })
}

pub fn get_unit_spawnpoints(
    player_id: u64,
    static_game_state: &StaticGameState,
    dynamic_game_state: &DynamicGameState,
) -> Vec<UnitSpawnpointTarget> {
    let direction = dynamic_game_state
        .players
        .get(&player_id)
        .unwrap()
        .direction
        .clone();
    dynamic_game_state
        .entities
        .iter()
        .filter(|(_id, entity)| entity.owner == player_id && entity.usable_as_spawn_point)
        .filter_map(|(_id, entity)| get_closest_path_point(&static_game_state.path, entity))
        .map(|(path_id, path_pos)| UnitSpawnpointTarget {
            path_id,
            direction: direction.clone(),
            path_pos,
        })
        .collect()
}
