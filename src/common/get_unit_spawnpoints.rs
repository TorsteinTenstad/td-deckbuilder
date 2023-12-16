use macroquad::math::Vec2;

use crate::{level_config, play_target::UnitSpawnpointTarget, DynamicGameState, StaticGameState};

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
        .filter_map(|(_id, entity)| {
            (entity.owner == player_id && entity.usable_as_spawn_point).then_some(entity.pos)
        })
        .flat_map(|pos| {
            static_game_state
                .paths
                .iter()
                .filter_map(move |(path_id, path)| {
                    path.iter()
                        .enumerate()
                        .map(|(path_pos, (x, y))| {
                            (
                                *path_id,
                                path_pos,
                                path_pos as f32 + (pos - Vec2::new(*x, *y)).length(),
                            )
                        })
                        .min_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap())
                        .filter(|(_, _, dist)| dist < &level_config::SPAWN_POINT_RADIUS)
                        .map(|(path_id, path_pos, _)| (path_id, path_pos))
                })
        })
        .map(|(path_id, path_pos)| UnitSpawnpointTarget {
            path_id,
            direction: direction.clone(),
            path_pos: path_pos as f32,
        })
        .collect()
}
