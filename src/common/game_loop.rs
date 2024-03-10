use crate::{
    buff::buff_update_timers, component_attack::Attack, component_buff_aura::BuffAura,
    component_health::Health, component_movement::Movement, config::CLOSE_ENOUGH_TO_TARGET,
    entity::EntityState, find_target::find_target_for_attack, game_state::ServerControledGameState,
    ids::EntityId, update_args::UpdateArgs, world::world_place_building,
};

fn cleanup_entity(
    entity_id: EntityId,
    server_controlled_game_state: &mut ServerControledGameState,
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

pub fn update_game_state(server_controlled_game_state: &mut ServerControledGameState, dt: f32) {
    //TODO: This implementation may cause entities to not be updated if the update_entities directly removes entities.
    // This could be solved by cashing the update state of all entities, or by only killing entities by setting their state to dead.
    let mut i = 0;
    while i < server_controlled_game_state
        .dynamic_game_state
        .entities
        .len()
    {
        let mut entity_instance = server_controlled_game_state
            .dynamic_game_state
            .entities
            .swap_remove(i);
        update_entity(&mut UpdateArgs {
            static_game_state: &server_controlled_game_state.static_game_state,
            semi_static_game_state: &mut server_controlled_game_state.semi_static_game_state,
            dynamic_game_state: &mut server_controlled_game_state.dynamic_game_state,
            entity_instance: &mut entity_instance,
            dt,
        });
        // TODO: Inserting at i causes a lot of memory movement, this can be optimized using a better swap routine for updating.
        server_controlled_game_state
            .dynamic_game_state
            .entities
            .insert(i, entity_instance);
        i += 1;
    }

    let mut i = 0;
    while i < server_controlled_game_state
        .dynamic_game_state
        .entities
        .len()
    {
        let entity = &server_controlled_game_state
            .dynamic_game_state
            .entities
            .get(i)
            .unwrap();
        if entity.state == EntityState::Dead {
            cleanup_entity(entity.id, server_controlled_game_state);
            server_controlled_game_state
                .dynamic_game_state
                .entities
                .swap_remove(i);
        } else {
            i += 1;
        }
    }
}

pub fn update_entity(update_args: &mut UpdateArgs) {
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
                attack
                    .range
                    .to_f32(update_args.entity_instance.entity.radius),
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

    buff_update_timers(&mut update_args.entity_instance.entity, update_args.dt);
    BuffAura::update(update_args);
    Health::update(update_args);
}