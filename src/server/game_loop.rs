use common::{
    component_attack::Attack,
    component_movement::Movement,
    config::CLOSE_ENOUGH_TO_TARGET,
    entity::{Entity, EntityState},
    find_target::find_target_for_attack,
    game_state::{DynamicGameState, StaticGameState},
    world::world_place_building,
};

pub fn update_entity<'a>(
    entity: &mut Entity,
    static_state: &StaticGameState,
    dynamic_game_state: &mut DynamicGameState,
    dt: f32,
) {
    let can_attack = entity.attacks.iter().any(|attack| {
        find_target_for_attack(
            entity.id,
            entity.tag.clone(),
            entity.pos,
            entity.owner,
            entity.spy.as_ref(),
            attack.range.to_f32(entity.radius),
            attack,
            &mut dynamic_game_state.entities,
        )
        .is_some()
    });

    let can_build =
        entity
            .building_to_construct
            .clone()
            .is_some_and(|(building_spot_target, _)| {
                (dynamic_game_state
                    .building_locations
                    .get(&building_spot_target.id)
                    .unwrap()
                    .pos
                    - entity.pos)
                    .length()
                    < CLOSE_ENOUGH_TO_TARGET
            });

    if can_attack {
        entity.state = EntityState::Attacking;
    } else if can_build {
        entity.state = EntityState::Building;
    } else {
        entity.state = EntityState::Moving;
    }

    match entity.state {
        EntityState::Moving => {
            Movement::update(entity, dynamic_game_state, dt, static_state);
        }

        EntityState::Attacking => {
            Attack::update(entity, &mut dynamic_game_state.entities, dt);
        }

        EntityState::Building => {
            if let Some((building_spot_target, entity_blueprint)) =
                entity.building_to_construct.clone()
            {
                let building_to_construct_pos = dynamic_game_state
                    .building_locations
                    .get(&building_spot_target.id)
                    .unwrap()
                    .pos;
                if (building_to_construct_pos - entity.pos).length() < CLOSE_ENOUGH_TO_TARGET {
                    world_place_building(
                        dynamic_game_state,
                        entity_blueprint.create(entity.owner),
                        building_spot_target,
                    );
                    entity.state = EntityState::Dead;
                }
            } else {
                assert!(false);
            }
        }

        EntityState::Passive | EntityState::Dead => {}
    }

    entity.health.damage_animation -= dt;
    if let Some(seconds_left_to_live) = &mut entity.seconds_left_to_live {
        *seconds_left_to_live -= dt;
        if seconds_left_to_live < &mut 0.0 {
            entity.state = EntityState::Dead;
        }
    }
    if entity.health.health <= 0.0 && entity.health.damage_animation < 0.0 {
        entity.state = EntityState::Dead;
    }
}
