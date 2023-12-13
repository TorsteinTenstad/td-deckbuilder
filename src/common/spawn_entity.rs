use crate::{Entity, PlayTarget, ServerGameState};
use rand::Rng;

pub fn spawn_entity(
    server_game_state: &mut ServerGameState,
    entity: Entity,
) -> () {
    server_game_state
        .dynamic_state
        .entities
        .insert(rand::thread_rng().gen(), entity);
}
