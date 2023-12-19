use crate::{
    draw::{to_screen_x, to_screen_y, unit_spawnpoint_gui_indicator_transform},
    hand_try_play,
};
use common::{
    card::CardInstance,
    get_unit_spawnpoints::get_unit_spawnpoints,
    play_target::{BuildingSpotTarget, EntityTarget, PlayFn, WorldPosTarget},
    ClientCommand, PlayTarget,
};
use macroquad::{
    input::{is_mouse_button_pressed, is_mouse_button_released},
    math::Vec2,
    miniquad::MouseButton,
};

use crate::{
    draw::{
        card_transform, curser_is_inside, hovered_card_transform, out_of_hand_card_transform,
        RectTransform,
    },
    input::{mouse_position_vec, mouse_world_position},
    ClientGameState,
};

pub struct PhysicalCard {
    pub card_instance: CardInstance,
    pub transform: RectTransform,
    pub target_transform: RectTransform,
}

impl PhysicalCard {
    pub fn new(card_instance: CardInstance) -> Self {
        Self {
            card_instance,
            transform: RectTransform::default(),
            target_transform: RectTransform::default(),
        }
    }
}

pub fn player_step(state: &mut ClientGameState) {
    let hand_size = state.physical_hand.cards.len();
    let mut top_hovering_card_idx: Option<usize> = None;
    state.unit_spawnpoint_targets.clear();

    if let Some(card_idx_being_held) = state.physical_hand.card_idx_being_held {
        let Vec2 { x, y } = mouse_position_vec();
        state
            .physical_hand
            .cards
            .get_mut(card_idx_being_held)
            .unwrap()
            .target_transform = out_of_hand_card_transform(x, y);

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
                        curser_is_inside(&unit_spawnpoint_gui_indicator_transform(
                            target,
                            &state.static_game_state,
                        ))
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
                            (mouse_position_vec() - Vec2 { x, y }).length() < r
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
            let in_hand_transform = card_transform(
                i,
                hand_size,
                state.relative_splay_radius,
                state.card_delta_angle,
            );
            if curser_is_inside(&in_hand_transform) {
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
                .target_transform = hovered_card_transform(
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
