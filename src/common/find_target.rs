use macroquad::math::Vec2;

use crate::{
    component_attack::{Attack, TargetPool},
    entity::{Entity, EntityTag, Spy},
    ids::{EntityId, PlayerId},
};

pub fn find_target_for_attack<'a>(
    entity_id: EntityId,
    entity_tag: EntityTag,
    entity_pos: Vec2,
    entity_owner: PlayerId,
    entity_spy: Option<&Spy>,
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
                    && can_find_target(entity_id, entity_tag.clone(), entity_spy, other_entity)
            },
        ),
        TargetPool::Allies => find_entity_in_range(
            entity_pos,
            range,
            &attack.can_target,
            other_entities,
            |other_entity| {
                other_entity.owner == entity_owner
                    && can_find_target(entity_id, entity_tag.clone(), entity_spy, other_entity)
            },
        ),
        TargetPool::All => find_entity_in_range(
            entity_pos,
            range,
            &attack.can_target,
            other_entities,
            |other_entity| can_find_target(entity_id, entity_tag.clone(), entity_spy, other_entity),
        ),
    }
}

pub fn can_find_target(
    entity_id: EntityId,
    entity_tag: EntityTag,
    entity_spy: Option<&Spy>,
    other_entity: &mut Entity,
) -> bool {
    if let Some(entity_spy) = entity_spy {
        if entity_spy.is_hidden()
            && other_entity.tag != EntityTag::Tower
            && other_entity.tag != EntityTag::Base
        {
            return false;
        }
    }
    let Some(spy) = other_entity.spy.as_mut() else {
        return true;
    };
    !spy.can_hide_from(entity_id, entity_tag)
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
        .filter(|other_entity| can_target.contains(&other_entity.tag))
        .filter(|other_entity| {
            (other_entity.pos - entity_pos).length_squared()
                < (range + other_entity.hitbox_radius).powi(2)
        })
        .filter_map(|x| filter_predicate(x).then(|| x))
        .min_by(|other_entity_a, other_entity_b| {
            let signed_distance_a = (other_entity_a.pos - entity_pos).length_squared()
                - (range + other_entity_a.hitbox_radius).powi(2);
            let signed_distance_b = (other_entity_b.pos - entity_pos).length_squared()
                - (range + other_entity_b.hitbox_radius).powi(2);
            signed_distance_a.partial_cmp(&signed_distance_b).unwrap()
        })
}
