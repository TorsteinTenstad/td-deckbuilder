use crate::{
    buff::buff_update_timers,
    component_attack::Attack,
    component_buff_source::BuffSource,
    component_health::Health,
    component_movement::Movement,
    config::CLOSE_ENOUGH_TO_TARGET,
    entities::{remove_dead_entities, update_entities},
    entity::EntityState,
    find_target::find_target_for_attack,
    game_state::ServerControlledGameState,
    ids::EntityId,
    update_args::UpdateArgs,
    world::world_place_building,
};

pub fn cleanup_entity(
    entity_id: EntityId,
    server_controlled_game_state: &mut ServerControlledGameState,
) {
    if let Some((_id, building_location)) = server_controlled_game_state
        .semi_static_game_state
        .building_locations_mut()
        .iter_mut()
        .find(|(_id, building_location)| building_location.entity_id == Some(entity_id))
    {
        building_location.entity_id = None;
    }
}

pub fn update_game_state(server_controlled_game_state: &mut ServerControlledGameState, dt: f32) {
    remove_dead_entities(server_controlled_game_state);
    for entity_instance in server_controlled_game_state
        .dynamic_game_state
        .entities
        .iter_mut()
    {
        buff_update_timers(&mut entity_instance.entity, dt);
        if entity_instance.state == EntityState::CreationFrame {
            entity_instance.state = EntityState::SpawnFrame;
        } else if entity_instance.state == EntityState::SpawnFrame {
            entity_instance.state = EntityState::Moving;
        }
    }
    update_entities(server_controlled_game_state, dt);
}

pub fn update_entity(update_args: &mut UpdateArgs) {
    BuffSource::update(update_args);

    let can_attack = update_args
        .entity_instance
        .entity
        .attacks
        .iter()
        .any(|attack| {
            find_target_for_attack(
                update_args.entity_instance.id,
                update_args.entity_instance.entity.tag.clone(),
                update_args.entity_instance.pos,
                update_args.entity_instance.owner,
                update_args.entity_instance.entity.spy.as_ref(),
                attack.get_range(update_args.entity_instance.entity.radius),
                attack,
                &mut update_args.dynamic_game_state.entities,
            )
            .is_some()
        });

    let can_build = update_args
        .entity_instance
        .entity
        .building_to_construct
        .clone()
        .is_some_and(|(building_spot_target, _)| {
            (update_args
                .semi_static_game_state
                .building_locations()
                .get(&building_spot_target.id)
                .unwrap()
                .pos
                - update_args.entity_instance.pos)
                .length()
                < CLOSE_ENOUGH_TO_TARGET
        });

    if can_attack {
        update_args.entity_instance.state = EntityState::Attacking;
    } else if can_build {
        update_args.entity_instance.state = EntityState::Building;
    } else {
        update_args.entity_instance.state = EntityState::Moving;
    }

    match update_args.entity_instance.state {
        EntityState::CreationFrame | EntityState::SpawnFrame => {} // State transitions are handled for all entities at once by update_game_state
        EntityState::Moving => {
            Movement::update(update_args);
        }

        EntityState::Attacking => {
            Attack::update(update_args);
        }

        EntityState::Building => {
            if let Some((building_spot_target, entity_blueprint)) = update_args
                .entity_instance
                .entity
                .building_to_construct
                .clone()
            {
                let building_to_construct_pos = update_args
                    .semi_static_game_state
                    .building_locations()
                    .get(&building_spot_target.id)
                    .unwrap()
                    .pos;
                if (building_to_construct_pos - update_args.entity_instance.pos).length()
                    < CLOSE_ENOUGH_TO_TARGET
                {
                    world_place_building(
                        update_args.semi_static_game_state,
                        update_args.dynamic_game_state,
                        entity_blueprint.create(),
                        &building_spot_target.id,
                        update_args.entity_instance.owner,
                    );
                    update_args.entity_instance.state = EntityState::Dead;
                }
            } else {
                debug_assert!(false);
            }
        }

        EntityState::Passive | EntityState::Dead => {}
    }

    Health::update(update_args);
}
