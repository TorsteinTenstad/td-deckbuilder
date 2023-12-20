use std::collections::HashMap;

use common::{
    component_attack::Attack,
    component_movement_behavior::MovementBehavior,
    entity::{Entity, EntityState},
    game_state::StaticGameState,
    ids::EntityId,
    world::{find_entity_in_range, BuildingLocation},
};

pub fn update_entity<'a>(
    entity: &mut Entity,
    other_entities: &mut Vec<Entity>,
    building_locations: &mut HashMap<BuildingLocationId, BuildingLocation>,
    dt: f32,
    static_state: &StaticGameState,
    new_entities: &mut Vec<Entity>,
    entity_ids_to_remove: &mut Vec<EntityId>,
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

        EntityState::Building => {
            if let Some(building_to_construct) = &entity.building_to_construct {
                let building = building_to_construct.1.create(entity.owner);
                let BuildingLocation { pos, entity_id } = building_locations
                    .get_mut(&building_to_construct.0)
                    .unwrap();
                entity.pos = pos;
                entity_id = Some(entity.id);
                new_entities.push(*entity);
            }
        }

        EntityState::Passive => {}
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
