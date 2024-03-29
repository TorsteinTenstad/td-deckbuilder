use macroquad::math::Vec2;

use crate::{
    component_attack::Attack,
    component_spy::Spy,
    entity::{EntityInstance, EntityTag},
    enum_flags::EnumFlags,
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
    other_entities: &'a mut [EntityInstance],
) -> Option<&'a mut EntityInstance> {
    find_entity_in_range(
        entity_pos,
        range,
        &attack.can_target,
        other_entities,
        |other_entity| {
            attack.target_pool.in_pool(entity_owner, other_entity.owner)
                && can_find_target(entity_id, entity_tag.clone(), entity_spy, other_entity)
        },
    )
}

pub fn can_find_target(
    entity_id: EntityId,
    entity_tag: EntityTag,
    entity_spy: Option<&Spy>,
    other_entity_instance: &mut EntityInstance,
) -> bool {
    if let Some(entity_spy) = entity_spy {
        if entity_spy.is_hidden()
            && other_entity_instance.entity.tag != EntityTag::Tower
            && other_entity_instance.entity.tag != EntityTag::Base
        {
            return false;
        }
    }
    let Some(spy) = other_entity_instance.entity.spy.as_mut() else {
        return true;
    };
    !spy.can_hide_from(entity_id, entity_tag)
}

pub fn find_entity_in_range<'a>(
    entity_pos: Vec2,
    range: f32,
    can_target: &EnumFlags<EntityTag>,
    other_entities: &'a mut [EntityInstance],
    filter_predicate: impl Fn(&mut EntityInstance) -> bool,
) -> Option<&'a mut EntityInstance> {
    other_entities
        .iter_mut()
        .filter(|other_entity_instance| can_target.is_set(&other_entity_instance.entity.tag))
        .filter(|other_entity_instance| {
            (other_entity_instance.pos - entity_pos).length_squared()
                < (range + other_entity_instance.entity.hitbox_radius).powi(2)
        })
        .filter_map(|x| filter_predicate(x).then_some(x))
        .min_by(|other_entity_instance_a, other_entity_instance_b| {
            let signed_distance_a = (other_entity_instance_a.pos - entity_pos).length_squared()
                - (range + other_entity_instance_a.entity.hitbox_radius).powi(2);
            let signed_distance_b = (other_entity_instance_b.pos - entity_pos).length_squared()
                - (range + other_entity_instance_b.entity.hitbox_radius).powi(2);
            signed_distance_a.partial_cmp(&signed_distance_b).unwrap()
        })
}
