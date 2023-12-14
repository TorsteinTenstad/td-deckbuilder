use crate::{DynamicGameState, Entity};
use rand::Rng;

pub fn spawn_entity(dynamic_game_state: &mut DynamicGameState, entity: Entity) -> () {
    dynamic_game_state
        .entities
        .insert(rand::thread_rng().gen(), entity);
}
