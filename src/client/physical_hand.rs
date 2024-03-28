use crate::{
    input::mouse_position_vec,
    physical_card::{
        card_transform_hovered, card_transform_in_hand, card_transform_outside_hand,
        PhysicalCardInstance,
    },
    ClientGameState,
};
use common::{
    card::CardInstance,
    get_unit_spawnpoints::get_unit_spawnpoints,
    network::ClientMessage,
    play_target::{
        unit_spawnpoint_target_transform, BuildingLocationTarget, EntityTarget, PlayFn, PlayTarget,
        WorldPosTarget,
    },
    rect_transform::point_inside,
};
use macroquad::{
    input::{is_mouse_button_pressed, is_mouse_button_released},
    math::Vec2,
    miniquad::MouseButton,
};

#[derive(Default)]
pub struct PhysicalHand {
    pub card_idx_being_held: Option<usize>,
    pub cards: Vec<PhysicalCardInstance>,
}

pub fn hand_sync(state: &mut ClientGameState) {
    // TODO: Find a way to remove clone?
    let server_hand = state.get_player().hand.cards.clone();
    for card_instance in server_hand.iter() {
        let physical_card = state
            .physical_hand
            .cards
            .iter_mut()
            .find(|c| c.card_instance.id == card_instance.id);
        if let Some(physical_card) = physical_card {
            physical_card.card_instance = card_instance.clone();
        } else {
            state
                .physical_hand
                .cards
                .push(PhysicalCardInstance::new(card_instance.clone()));
        }
    }
    state.physical_hand.cards.retain(|physical_card| {
        server_hand
            .iter()
            .any(|card_instance| card_instance.id == physical_card.card_instance.id)
    });
}

pub fn hand_try_play(state: &ClientGameState) -> Option<CardInstance> {
    let Some(card_idx_being_held) = state.physical_hand.card_idx_being_held else {
        return None;
    };
    let card_instance = state
        .physical_hand
        .cards
        .get(card_idx_being_held)
        .unwrap()
        .card_instance
        .clone();
    if state.get_player().hand.energy < card_instance.card.get_card_data().energy_cost {
        return None;
    }
    Some(card_instance)
}

pub fn hand_step(state: &mut ClientGameState) {
    let hand_size = state.physical_hand.cards.len();
    let mut top_hovering_card_idx: Option<usize> = None;

    if let Some(card_idx_being_held) = state.physical_hand.card_idx_being_held {
        let Vec2 { x, y } = mouse_position_vec();
        state
            .physical_hand
            .cards
            .get_mut(card_idx_being_held)
            .unwrap()
            .target_transform = card_transform_outside_hand(x, y);

        let card_data = state
            .physical_hand
            .cards
            .get(card_idx_being_held)
            .unwrap()
            .card_instance
            .card
            .get_card_data();

        if is_mouse_button_released(MouseButton::Left) {
            match card_data.play_fn {
                PlayFn::WorldPos(_) => {
                    if let Some(card_instance) = hand_try_play(state) {
                        let Vec2 { x, y } = mouse_position_vec();
                        state
                            .client_network_state
                            .push_command(ClientMessage::PlayCard(
                                card_instance.id,
                                PlayTarget::WorldPos(WorldPosTarget { x, y }),
                            ));
                    }
                }
                PlayFn::UnitSpawnPoint(_) => {
                    let unit_spawnpoint_targets = get_unit_spawnpoints(
                        state.player_id,
                        &state.server_controlled_game_state.static_game_state,
                        &state.server_controlled_game_state.dynamic_game_state,
                    );
                    if let Some(target) = unit_spawnpoint_targets.iter().find(|target| {
                        point_inside(
                            mouse_position_vec(),
                            &unit_spawnpoint_target_transform(
                                target,
                                &state.server_controlled_game_state.static_game_state,
                            ),
                        )
                    }) {
                        if let Some(card_instance) = hand_try_play(state) {
                            state
                                .client_network_state
                                .push_command(ClientMessage::PlayCard(
                                    card_instance.id,
                                    PlayTarget::UnitSpawnpoint(target.clone()),
                                ));
                        }
                    }
                }
                PlayFn::BuildingLocation(_) => {
                    if let Some((id, _pos)) = state
                        .server_controlled_game_state
                        .semi_static_game_state
                        .building_locations()
                        .iter()
                        .find(|(_, loc)| {
                            let r = 20.0;
                            (mouse_position_vec() - loc.pos).length() < r
                        })
                    {
                        if let Some(card_instance) = hand_try_play(state) {
                            state
                                .client_network_state
                                .push_command(ClientMessage::PlayCard(
                                    card_instance.id,
                                    PlayTarget::BuildingLocation(BuildingLocationTarget {
                                        id: *id,
                                    }),
                                ));
                        }
                    }
                }
                PlayFn::Entity(_) => {
                    if let Some(entity) = state
                        .server_controlled_game_state
                        .dynamic_game_state
                        .entities
                        .iter()
                        .find(|entity_instance| {
                            (entity_instance.pos - mouse_position_vec()).length()
                                < entity_instance.entity.radius
                        })
                    {
                        if let Some(card_instance) = hand_try_play(state) {
                            state
                                .client_network_state
                                .push_command(ClientMessage::PlayCard(
                                    card_instance.id,
                                    PlayTarget::Entity(EntityTarget { id: entity.id }),
                                ));
                        }
                    }
                }
            }
            state.physical_hand.card_idx_being_held = None;
        }
    } else {
        for (i, physical_card) in state.physical_hand.cards.iter_mut().enumerate() {
            let in_hand_transform = card_transform_in_hand(
                i,
                hand_size,
                state.relative_splay_radius,
                state.card_delta_angle,
            );
            if point_inside(mouse_position_vec(), &in_hand_transform) {
                top_hovering_card_idx = Some(i);
                if is_mouse_button_pressed(MouseButton::Left) {
                    state.physical_hand.card_idx_being_held = Some(i);
                }
            }
            physical_card.target_transform = in_hand_transform;
        }
        if let Some(i) = top_hovering_card_idx {
            state
                .physical_hand
                .cards
                .get_mut(i)
                .unwrap()
                .target_transform = card_transform_hovered(
                i,
                hand_size,
                state.relative_splay_radius,
                state.card_delta_angle,
            );
        }
    }

    for physical_card in state.physical_hand.cards.iter_mut() {
        physical_card
            .transform
            .animate_towards(&physical_card.target_transform, state.dt * 20.0);
    }
}
