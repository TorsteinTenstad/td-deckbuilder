use macroquad::math::Vec2;

use crate::{
    component_attack::{Attack, TargetPool},
    entity::{Entity, EntityTag},
    ids::{EntityId, PlayerId},
};

pub fn find_attack_targets<'a>(
    entity: &'a mut Entity,
    other_entities: &'a mut Vec<Entity>,
    range_override: Option<f32>,
) -> Option<&'a mut Entity> {
    let attack = entity.attacks.first()?;
    find_target_for_attack(
        entity.id,
        &entity.tag,
        entity.pos,
        entity.owner,
        range_override.unwrap_or(attack.get_range(entity.radius)),
        attack,
        other_entities,
    )
}

pub fn find_target_for_attack<'a>(
    entity_id: EntityId,
    entity_tag: &EntityTag,
    entity_pos: Vec2,
    entity_owner: PlayerId,
    range: f32,
    attack: &Attack,
    other_entities: &'a mut Vec<Entity>,
) -> Option<&'a mut Entity> {
    match attack.target_pool {
        TargetPool::Enemies => find_entity_in_range(
            entity_pos,
            range,
            &attack.can_target,
            other_entities,
            |other_entity| {
                other_entity.owner != entity_owner
                    && can_find_target(entity_id, entity_tag, other_entity)
            },
        ),
        TargetPool::Allies => find_entity_in_range(
            entity_pos,
            range,
            &attack.can_target,
            other_entities,
            |other_entity| other_entity.owner == entity_owner,
        ),
        TargetPool::All => find_entity_in_range(
            entity_pos,
            range,
            &attack.can_target,
            other_entities,
            |other_entity| can_find_target(entity_id, entity_tag, other_entity),
        ),
    }
}

pub fn can_find_target(entity_id: EntityId, tag: &EntityTag, other_entity: &mut Entity) -> bool {
    let Some(spy) = other_entity.spy.as_mut() else {
        return true;
    };
    !spy.can_hide_from(&entity_id, tag)
}

pub fn find_entity_in_range<'a>(
    entity_pos: Vec2,
    range: f32,
    can_target: &Vec<EntityTag>,
    other_entities: &'a mut Vec<Entity>,
    filter_predicate: impl Fn(&mut Entity) -> bool,
) -> Option<&'a mut Entity> {
    other_entities
        .iter_mut()
        .filter_map(|x| filter_predicate(x).then(|| x))
        .filter(|other_entity| can_target.contains(&other_entity.tag))
        .filter(|other_entity| {
            (other_entity.pos - entity_pos).length_squared()
                < (range + other_entity.hitbox_radius).powi(2)
        })
        .min_by(|other_entity_a, other_entity_b| {
            let signed_distance_a = (other_entity_a.pos - entity_pos).length_squared()
                - (range + other_entity_a.hitbox_radius).powi(2);
            let signed_distance_b = (other_entity_b.pos - entity_pos).length_squared()
                - (range + other_entity_b.hitbox_radius).powi(2);
            signed_distance_a.partial_cmp(&signed_distance_b).unwrap()
        })
}
