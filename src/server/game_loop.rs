use common::{
    component_attack::Attack,
    component_movement_behavior::MovementBehavior,
    entity::{Entity, EntityState},
    find_target::find_target_for_attack,
    game_state::{DynamicGameState, StaticGameState},
};

pub fn update_entity<'a>(
    entity: &mut Entity,
    static_state: &StaticGameState,
    dynamic_game_state: &mut DynamicGameState,
    dt: f32,
) {
    let can_attack = entity.attacks.iter().any(|attack| {
        find_target_for_attack(
            entity.pos,
            entity.owner,
            attack.range,
            attack,
            &mut dynamic_game_state.entities,
        )
        .is_some()
    });

    match entity.state {
        EntityState::Moving => {
            MovementBehavior::update(entity, dynamic_game_state, dt, static_state);
            if can_attack {
                entity.state = EntityState::Attacking;
            }
        }

        EntityState::Attacking => {
            Attack::update(entity, &mut dynamic_game_state.entities, dt);
            if !can_attack {
                entity.state = EntityState::Moving
            }
        }

        EntityState::Building => {
            if let Some((building_location_id, entity_blueprint)) = &entity.building_to_construct {
                let mut building = entity_blueprint.create(entity.owner);
                let building_location = dynamic_game_state
                    .building_locations
                    .get_mut(&building_location_id.id)
                    .unwrap();
                building.pos = building_location.pos;
                building_location.entity_id = Some(building.id);
                dynamic_game_state.entities.push(building);
            }
        }

        EntityState::Passive | EntityState::Dead => {}
    }

    entity.damage_animation -= dt;
    if let Some(seconds_left_to_live) = &mut entity.seconds_left_to_live {
        *seconds_left_to_live -= dt;
        if seconds_left_to_live < &mut 0.0 {
            entity.health = 0.0;
        }
    }
    if entity.health <= 0.0 && entity.damage_animation < 0.0 {
        entity.state = EntityState::Dead;
    }
}
