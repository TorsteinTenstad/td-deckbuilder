use std::collections::HashMap;

use crate::Entity;
use rand::Rng;

pub fn spawn_entity(entities: &mut HashMap<u64, Entity>, entity: Entity) -> u64 {
    // TODO, Magne: uuid?
    let key = rand::thread_rng().gen();

    entities.insert(key, entity);

    return key;
}
