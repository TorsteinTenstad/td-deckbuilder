use common::{
    melee_attack::MeleeAttack, ranged_attack::RangedAttack, Entity, EntityState, MovementBehavior,
    StaticGameState,
};

pub fn update_entity<'a>(
    entity: &mut Entity,
    other_entities: &mut Vec<Entity>,
    dt: f32,
    static_state: &StaticGameState,
    new_entities: &mut Vec<Entity>,
    entity_ids_to_remove: &mut Vec<u64>,
) {
    match entity.state {
        EntityState::Moving => {
            MovementBehavior::update(entity, other_entities, dt, static_state);
        }

        EntityState::Attacking => {
            RangedAttack::update(entity, other_entities, dt, new_entities);
            MeleeAttack::update(entity, other_entities, dt);
        }
    }

    entity.damage_animation -= dt;
    if let Some(seconds_left_to_live) = &mut entity.seconds_left_to_live {
        *seconds_left_to_live -= dt;
        if seconds_left_to_live < &mut 0.0 {
            entity.health = 0.0;
        }
    }
    if entity.health < 0.0 {
        entity_ids_to_remove.push(entity.id);
    }
}
