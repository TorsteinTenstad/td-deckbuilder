use common::{
    attack::Attack,
    component_movement_behavior::MovementBehavior,
    entity::{Entity, EntityState},
    game_state::StaticGameState,
    world::find_entity_in_range,
};

pub fn update_entity<'a>(
    entity: &mut Entity,
    other_entities: &mut Vec<Entity>,
    dt: f32,
    static_state: &StaticGameState,
    new_entities: &mut Vec<Entity>,
    entity_ids_to_remove: &mut Vec<u64>,
) {
    let can_attack = entity.attacks.iter().any(|attack| {
        find_entity_in_range(
            entity.pos,
            entity.owner,
            attack.range,
            &attack.can_target,
            other_entities,
        )
        .is_some()
    });

    match entity.state {
        EntityState::Moving => {
            MovementBehavior::update(entity, other_entities, dt, static_state);
            if can_attack {
                entity.state = EntityState::Attacking;
            }
        }

        EntityState::Attacking => {
            Attack::update(entity, other_entities, dt, new_entities);
            if !can_attack {
                entity.state = EntityState::Moving
            }
        }
    }

    entity.damage_animation -= dt;
    if let Some(seconds_left_to_live) = &mut entity.seconds_left_to_live {
        *seconds_left_to_live -= dt;
        if seconds_left_to_live < &mut 0.0 {
            entity.health = 0.0;
        }
    }
    if entity.health <= 0.0 && entity.damage_animation < 0.0 {
        entity_ids_to_remove.push(entity.id);
    }
}
