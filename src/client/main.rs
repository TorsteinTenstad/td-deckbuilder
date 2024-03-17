use client_game_state::ClientGameState;
use common::component_attack::{Attack, AttackVariant};
use common::component_movement::get_detection_range;
use common::draw::{
    draw_card, draw_progress_bar, draw_rect_transform, to_screen_size, to_screen_transform,
    to_screen_x, to_screen_y, Sprites,
};
use common::draw_server_controlled_game_state::draw_server_controlled_game_state;
use common::game_state::{DynamicGameState, ServerControlledGameState, StaticGameState};
use common::get_unit_spawnpoints::get_unit_spawnpoints;
use common::ids::{EntityId, PlayerId};
use common::network::ClientMessage;
use common::play_target::{unit_spawnpoint_target_transform, BuildingSpotTarget, PlayFn};
use common::rect_transform::{point_inside, RectTransform};
use common::textures::SpriteId;
use common::world::{find_entity, Zoning};
use input::{main_input, mouse_screen_position, mouse_world_position};
use itertools::Itertools;
use macroquad::color::{Color, BLACK, BLUE, RED, WHITE, YELLOW};
use macroquad::input::is_key_pressed;
use macroquad::math::Vec2;
use macroquad::miniquad::KeyCode;
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_poly_lines};
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
use macroquad::window::{clear_background, screen_height, screen_width};
use macroquad::{window::next_frame, window::request_new_screen_size};
use physical_hand::{hand_step, hand_sync, PhysicalHand};
use text_box::TextBox;
mod client_game_state;
pub mod config;
mod deck_builder;
mod hit_numbers;
mod input;
mod network;
mod physical_card;
mod physical_hand;
mod text_box;

#[macroquad::main("Client")]
async fn main() {
    request_new_screen_size(1280.0, 720.0);

    let mut state = ClientGameState::new().await;
    let mut text_box = TextBox::new(RectTransform {
        w: 200.0,
        h: 50.0,
        ..Default::default()
    });
    text_box.text = state.client_network_state.server_addr.to_string();

    loop {
        if state.in_deck_builder {
            state.step();
            state.deck_builder.step(state.dt);
            text_box.step();
            text_box.transform.x = screen_width() - text_box.transform.w - 10.0;
            text_box.transform.y = screen_height() - text_box.transform.h - 10.0;
            state.deck_builder.draw(&state.sprites);
            text_box.draw(Some(&state.font));

            next_frame().await;

            if is_key_pressed(KeyCode::Enter) {
                state.client_network_state.server_addr = text_box
                    .text
                    .parse()
                    .unwrap_or(state.client_network_state.server_addr);
                state.deck_builder.save();
                state.in_deck_builder = false;
            }
        } else {
            state
                .client_network_state
                .ensure_joined(ClientMessage::JoinGame(
                    state
                        .deck_builder
                        .deck
                        .iter()
                        .map(|physical_card| physical_card.card.clone())
                        .collect_vec(),
                ));
            while let Some(server_message) = state.client_network_state.receive() {
                let updated = state
                    .server_controlled_game_state
                    .update_with_server_message(server_message);
                if updated {
                    hand_sync(&mut state);
                }
            }
            main_step(&mut state);
            state.client_network_state.send_queued();
            draw_client_game_state(&state);

            next_frame().await;

            if is_key_pressed(KeyCode::Escape) {
                state.in_deck_builder = true;
            }
        }
    }
}

fn main_step(state: &mut ClientGameState) {
    state.step();
    main_input(state);
    hand_step(state);
    state.hit_numbers.step(
        &state
            .server_controlled_game_state
            .dynamic_game_state
            .entities,
        state.dt,
    );
}

fn draw_physical_hand(physical_hand: &PhysicalHand, sprites: &Sprites) {
    let mut physical_cards = physical_hand.cards.clone();
    physical_cards.sort_by(|a, b| {
        a.transform
            .w
            .partial_cmp(&b.transform.w)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for physical_card in physical_cards.iter() {
        draw_card(
            &physical_card.card_instance.card,
            &physical_card.transform,
            1.0,
            &sprites,
        )
    }
}

fn draw_building_location_play_targets(
    server_controlled_game_state: &ServerControlledGameState,
    physical_hand: &PhysicalHand,
    player_id: PlayerId,
) {
    for (id, loc) in server_controlled_game_state
        .semi_static_game_state
        .building_locations()
        .iter()
    {
        if physical_hand
            .card_idx_being_held
            .filter(|idx| {
                if let PlayFn::BuildingSpot(specific_play_fn) = &physical_hand
                    .cards
                    .get(*idx)
                    .unwrap()
                    .card_instance
                    .card
                    .get_card_data()
                    .play_fn
                {
                    !specific_play_fn.target_is_invalid.is_some_and(|f| {
                        f(
                            &BuildingSpotTarget { id: *id },
                            player_id,
                            &server_controlled_game_state.static_game_state,
                            &server_controlled_game_state.semi_static_game_state,
                            &server_controlled_game_state.dynamic_game_state,
                        )
                    })
                } else {
                    false
                }
            })
            .is_some()
        {
            let x = to_screen_x(loc.pos.x);
            let y = to_screen_y(loc.pos.y);
            let (poly_sides, radius) = match loc.zoning {
                Zoning::Normal => (20, 15.0),
                Zoning::Commerce => (6, 20.0),
            };
            let hovering = (mouse_screen_position() - Vec2 { x, y }).length() < radius;
            let thickness = 3.0;
            let color = Color {
                a: if hovering { 0.8 } else { 0.5 },
                ..RED
            };
            draw_poly_lines(x, y, poly_sides, radius, 0., thickness, color);
        }
    }
}

fn draw_range_circle_preview(
    dynamic_game_state: &DynamicGameState,
    selected_entity_id: Option<EntityId>,
) {
    let mut range_circle_preview: Vec<(f32, f32, f32, Color)> = Vec::new();
    if let Some(entity_instance) = find_entity(&dynamic_game_state.entities, selected_entity_id) {
        if let Some(Attack { range, .. }) = entity_instance
            .entity
            .attacks
            .iter()
            .find(|attack| attack.variant == AttackVariant::RangedAttack)
        {
            range_circle_preview.push((
                entity_instance.pos.x,
                entity_instance.pos.y,
                range.to_f32(entity_instance.entity.radius),
                BLUE,
            ));
        }

        if let Some(detection_range) = get_detection_range(&entity_instance.entity) {
            range_circle_preview.push((
                entity_instance.pos.x,
                entity_instance.pos.y,
                detection_range,
                YELLOW,
            ));
        }

        range_circle_preview.push((
            entity_instance.pos.x,
            entity_instance.pos.y,
            entity_instance.entity.hitbox_radius,
            RED,
        ));
    }
    for (x, y, range, color) in range_circle_preview {
        let x = to_screen_x(x);
        let y = to_screen_y(y);
        let r = to_screen_size(range);

        draw_circle(x, y, r, Color { a: 0.2, ..color });
        draw_circle_lines(x, y, r, 2.0, color);
    }
}

fn draw_progress_bars(state: &ClientGameState) {
    if let Some(player) = state
        .server_controlled_game_state
        .dynamic_game_state
        .players
        .get(&state.player_id)
    {
        let margin = 10.0;
        let outline_w = 5.0;
        let w = 25.0;
        let h = 100.0;
        let draw_progress = player.hand.card_draw_counter;
        draw_progress_bar(
            screen_width() - w - margin,
            screen_height() - h - margin,
            w,
            h,
            outline_w,
            draw_progress,
            state.physical_hand.cards.len() as i32,
            YELLOW,
            WHITE,
            BLACK,
            Some(&state.font),
        );
        let energy_progress = player.hand.energy_counter;
        draw_progress_bar(
            screen_width() - 2.0 * w - 2.0 * margin,
            screen_height() - h - margin,
            w,
            h,
            outline_w,
            energy_progress,
            state
                .server_controlled_game_state
                .dynamic_game_state
                .players
                .get(&state.player_id)
                .unwrap()
                .hand
                .energy,
            BLUE,
            WHITE,
            BLACK,
            Some(&state.font),
        );
    }
}

fn draw_spawnpoint_play_targets(
    player_id: PlayerId,
    static_game_state: &StaticGameState,
    dynamic_game_state: &DynamicGameState,
) {
    let unit_spawnpoint_targets =
        get_unit_spawnpoints(player_id, &static_game_state, &dynamic_game_state);
    for target in unit_spawnpoint_targets.iter() {
        let transform = &unit_spawnpoint_target_transform(target, &static_game_state);
        let hovering = point_inside(mouse_world_position(), transform);
        draw_rect_transform(
            &to_screen_transform(transform),
            Color {
                a: if hovering { 0.8 } else { 0.5 },
                ..RED
            },
        );
    }
}

fn draw_client_game_state(state: &ClientGameState) {
    if !state.has_player() {
        return;
    }

    // board
    clear_background(BLACK);
    draw_texture_ex(
        state.sprites.get_texture(&SpriteId::Map),
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2 {
                x: screen_width(),
                y: screen_height(),
            }),
            ..Default::default()
        },
    );

    draw_physical_hand(&state.physical_hand, &state.sprites);
    draw_server_controlled_game_state(
        &state.server_controlled_game_state,
        &state.sprites,
        &state.debug_draw_config,
    );
    draw_range_circle_preview(
        &state.server_controlled_game_state.dynamic_game_state,
        state.selected_entity_id,
    );
    draw_progress_bars(state);
    draw_building_location_play_targets(
        &state.server_controlled_game_state,
        &state.physical_hand,
        state.player_id,
    );
    draw_spawnpoint_play_targets(
        state.player_id,
        &state.server_controlled_game_state.static_game_state,
        &state.server_controlled_game_state.dynamic_game_state,
    );

    state.hit_numbers.draw(Some(&state.font));
}
