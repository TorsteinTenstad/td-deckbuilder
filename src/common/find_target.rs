use macroquad::math::Vec2;

use crate::{
    component_attack::{Attack, AttackVariant},
    entity::{Entity, EntityTag},
    ids::PlayerId,
};

pub fn find_target_for_attack<'a>(
    entity_pos: Vec2,
    entity_owner: PlayerId,
    range: f32,
    attack: &Attack,
    other_entities: &'a mut Vec<Entity>,
) -> Option<&'a mut Entity> {
    match attack.variant {
        AttackVariant::MeleeAttack | AttackVariant::RangedAttack => find_enemy_entity_in_range(
            entity_pos,
            entity_owner,
            range,
            &attack.can_target,
            other_entities,
        ),
        AttackVariant::Heal => find_friendly_entity_in_range(
            entity_pos,
            entity_owner,
            range,
            &attack.can_target,
            other_entities,
        ),
    }
}

pub fn find_enemy_entity_in_range<'a>(
    entity_pos: Vec2,
    entity_owner: PlayerId,
    range: f32,
    can_target: &Vec<EntityTag>,
    other_entities: &'a mut Vec<Entity>,
) -> Option<&'a mut Entity> {
    find_entity_in_range(
        entity_pos,
        range,
        other_entities,
        can_target,
        |other_entity| other_entity.owner != entity_owner,
    )
}

pub fn find_friendly_entity_in_range<'a>(
    entity_pos: Vec2,
    entity_owner: PlayerId,
    range: f32,
    can_target: &Vec<EntityTag>,
    other_entities: &'a mut Vec<Entity>,
) -> Option<&'a mut Entity> {
    find_entity_in_range(
        entity_pos,
        range,
        other_entities,
        can_target,
        |other_entity| other_entity.owner == entity_owner,
    )
}

pub fn find_entity_in_range<'a>(
    entity_pos: Vec2,
    range: f32,
    other_entities: &'a mut Vec<Entity>,
    can_target: &Vec<EntityTag>,
    filter_predicate: impl Fn(&&mut Entity) -> bool,
) -> Option<&'a mut Entity> {
    other_entities
        .iter_mut()
        .filter(filter_predicate)
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
