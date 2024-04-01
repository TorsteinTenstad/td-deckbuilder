use macroquad::math::Vec2;

use crate::{
    component_attack::Attack,
    component_spy::Spy,
    entity::{EntityInstance, EntityTag},
    enum_flags::EnumFlags,
    ids::{EntityId, PlayerId},
};

pub fn find_target_id_for_attack(
    entity_id: EntityId,
    entity_tag: EntityTag,
    entity_pos: Vec2,
    entity_owner: PlayerId,
    entity_spy: Option<&Spy>,
    range: f32,
    attack: &Attack,
    other_entities: &mut [EntityInstance],
) -> Option<EntityId> {
    find_targets_for_attack(
        entity_id,
        entity_tag,
        entity_pos,
        entity_owner,
        entity_spy,
        range,
        attack,
        other_entities,
    )
    .first()
    .map(|e| e.id)
}

pub fn find_target_ids_for_attack(
    entity_id: EntityId,
    entity_tag: EntityTag,
    entity_pos: Vec2,
    entity_owner: PlayerId,
    entity_spy: Option<&Spy>,
    range: f32,
    attack: &Attack,
    other_entities: &mut [EntityInstance],
) -> Vec<EntityId> {
    find_targets_for_attack(
        entity_id,
        entity_tag,
        entity_pos,
        entity_owner,
        entity_spy,
        range,
        attack,
        other_entities,
    )
    .iter()
    .map(|e| e.id)
    .collect()
}

pub fn find_targets_for_attack<'a>(
    entity_id: EntityId,
    entity_tag: EntityTag,
    entity_pos: Vec2,
    entity_owner: PlayerId,
    entity_spy: Option<&'a Spy>,
    range: f32,
    attack: &Attack,
    other_entities: &'a mut [EntityInstance],
) -> Vec<&'a mut EntityInstance> {
    let attack_target_pool = attack.target_pool.clone();
    find_entities_in_range(
        entity_pos,
        range,
        attack.can_target.clone(),
        other_entities,
        move |other_entity| {
            attack_target_pool.in_pool(entity_owner, other_entity.owner)
                && can_find_target(entity_id, entity_tag.clone(), entity_spy, other_entity)
        },
    )
}

fn can_find_target(
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

pub fn find_entities_in_range<'a>(
    source_pos: Vec2,
    range: f32,
    can_target: EnumFlags<EntityTag>,
    other_entities: &'a mut [EntityInstance],
    filter_predicate: impl Fn(&mut EntityInstance) -> bool + 'a,
) -> Vec<&'a mut EntityInstance> {
    let mut enities: Vec<&'a mut EntityInstance> = other_entities
        .iter_mut()
        .filter(move |other_entity_instance| can_target.is_set(&other_entity_instance.entity.tag))
        .filter(move |other_entity_instance| {
            (other_entity_instance.pos - source_pos).length_squared()
                < (range + other_entity_instance.entity.hitbox_radius).powi(2)
        })
        .filter_map(move |x| filter_predicate(x).then_some(x))
        .collect();
    enities.sort_by(|other_entity_instance_a, other_entity_instance_b| {
        let signed_distance_a = (other_entity_instance_a.pos - source_pos).length_squared()
            - (range + other_entity_instance_a.entity.hitbox_radius).powi(2);
        let signed_distance_b = (other_entity_instance_b.pos - source_pos).length_squared()
            - (range + other_entity_instance_b.entity.hitbox_radius).powi(2);
        signed_distance_a.partial_cmp(&signed_distance_b).unwrap()
    });
    enities
}
