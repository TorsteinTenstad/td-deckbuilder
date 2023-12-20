use crate::{
    draw::{to_screen_x, to_screen_y},
    input::{mouse_screen_position, mouse_world_position},
    physical_card::{
        card_transform_hovered, card_transform_in_hand, card_transform_outside_hand, PhysicalCard,
    },
    ClientGameState,
};
use common::{
    card::CardInstance,
    get_unit_spawnpoints::get_unit_spawnpoints,
    play_target::{
        unit_spawnpoint_target_transform, BuildingSpotTarget, EntityTarget, PlayFn, WorldPosTarget,
    },
    rect::point_inside,
    ClientCommand, PlayTarget,
};
use macroquad::{
    input::{is_mouse_button_pressed, is_mouse_button_released},
    math::Vec2,
    miniquad::MouseButton,
};

#[derive(Default)]
pub struct PhysicalHand {
    pub card_idx_being_held: Option<usize>,
    pub cards: Vec<PhysicalCard>,
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
                .push(PhysicalCard::new(card_instance.clone()));
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
    if state.get_player().hand.energy < card_instance.card.energy_cost() {
        return None;
    }
    Some(card_instance)
}

pub fn hand_step(state: &mut ClientGameState) {
    let hand_size = state.physical_hand.cards.len();
    let mut top_hovering_card_idx: Option<usize> = None;
    state.unit_spawnpoint_targets.clear();

    if let Some(card_idx_being_held) = state.physical_hand.card_idx_being_held {
        let Vec2 { x, y } = mouse_screen_position();
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
        match card_data.play_fn {
            PlayFn::WorldPos(_) => {}
            PlayFn::UnitSpawnPoint(_) => {
                state.unit_spawnpoint_targets = get_unit_spawnpoints(
                    state.player_id,
                    &state.static_game_state,
                    &state.dynamic_game_state,
                )
            }
            PlayFn::BuildingSpot(_) => {}
            PlayFn::Entity(_) => {}
        }

        if is_mouse_button_released(MouseButton::Left) {
            match card_data.play_fn {
                PlayFn::WorldPos(_) => {
                    if let Some(card_instance) = hand_try_play(state) {
                        let Vec2 { x, y } = mouse_world_position();
                        state.commands.push(ClientCommand::PlayCard(
                            card_instance.id,
                            PlayTarget::WorldPos(WorldPosTarget { x, y }),
                        ));
                    }
                }
                PlayFn::UnitSpawnPoint(_) => {
                    if let Some(target) = state.unit_spawnpoint_targets.iter().find(|target| {
                        point_inside(
                            mouse_world_position(),
                            &unit_spawnpoint_target_transform(target, &state.static_game_state),
                        )
                    }) {
                        if let Some(card_instance) = hand_try_play(state) {
                            state.commands.push(ClientCommand::PlayCard(
                                card_instance.id,
                                PlayTarget::UnitSpawnPoint(target.clone()),
                            ));
                        }
                    }
                }
                PlayFn::BuildingSpot(_) => {
                    if let Some((id, _pos)) = state
                        .dynamic_game_state
                        .building_locations
                        .iter()
                        .find(|(_, loc)| {
                            let x = to_screen_x(loc.position.0);
                            let y = to_screen_y(loc.position.1);
                            let r = 20.0;
                            (mouse_screen_position() - Vec2 { x, y }).length() < r
                        })
                    {
                        if let Some(card_instance) = hand_try_play(state) {
                            state.commands.push(ClientCommand::PlayCard(
                                card_instance.id,
                                PlayTarget::BuildingSpot(BuildingSpotTarget { id: *id }),
                            ));
                        }
                    }
                }
                PlayFn::Entity(_) => {
                    if let Some(entity) = state.dynamic_game_state.entities.iter().find(|entity| {
                        (entity.pos - mouse_world_position()).length() < entity.radius
                    }) {
                        if let Some(card_instance) = hand_try_play(state) {
                            state.commands.push(ClientCommand::PlayCard(
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
            if point_inside(mouse_screen_position(), &in_hand_transform) {
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
