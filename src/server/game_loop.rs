use std::collections::HashMap;

use common::{
    melee_attack::MeleeAttack, ranged_attack::RangedAttack, Entity, EntityExternalEffects,
    EntityState, MovementBehavior, StaticGameState,
};

pub fn update_entity(
    id: &u64,
    entity: &Entity,
    other_entities: &HashMap<u64, Entity>,
    other_entities_external_effects: &mut HashMap<u64, EntityExternalEffects>,
    dt: f32,
    static_game_state: &StaticGameState,
    rng: &mut impl rand::Rng,
) -> Vec<(u64, Entity)> {
    let mut new_entities = Vec::new();
    let mut entity = entity.clone();

    match entity.state {
        EntityState::Moving => {
            MovementBehavior::update(
                id,
                &mut entity,
                other_entities,
                other_entities_external_effects,
                dt,
                static_game_state,
                rng,
            );
        }

        EntityState::Attacking => {
            RangedAttack::update(
                id,
                &mut entity,
                other_entities,
                other_entities_external_effects,
                dt,
                static_game_state,
                rng,
                &mut new_entities,
            );
            MeleeAttack::update(
                id,
                &mut entity,
                other_entities,
                other_entities_external_effects,
                dt,
                static_game_state,
                rng,
            );
        }
    }

    entity.damage_animation -= dt;
    if let Some(seconds_left_to_live) = &mut entity.seconds_left_to_live {
        *seconds_left_to_live -= dt;
        if seconds_left_to_live < &mut 0.0 {
            entity.health = 0.0;
        }
    }
    if entity.health > 0.0 {
        new_entities.push((id.clone(), entity));
    }
    new_entities
}
