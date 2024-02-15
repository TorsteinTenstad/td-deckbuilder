use crate::{
    draw::{draw_card, load_sprites, Sprites, GOLDEN_RATIO},
    hit_numbers::HitNumbers,
    input::mouse_screen_position,
    physical_card::PhysicalCard,
    physical_hand::{hand_sync, PhysicalHand},
    ClientNetworkState,
};
use common::{
    card::Card,
    game_state::ServerControledGameState,
    ids::{EntityId, PlayerId},
    network::{ServerMessage, ServerMessageData},
    play_target::UnitSpawnpointTarget,
    rect_transform::{point_inside, RectTransform},
    server_player::ServerPlayer,
    vector::pop_where,
};
use itertools::Itertools;
use macroquad::{
    input::{is_mouse_button_pressed, is_mouse_button_released},
    math::Vec2,
    miniquad::MouseButton,
    text::Font,
    window::screen_width,
};
use std::time::SystemTime;

pub struct DeckBuilder {
    pub card_pool: Vec<PhysicalCard>,
    pub deck: Vec<PhysicalCard>,
    pub holding: Option<PhysicalCard>,
}

impl DeckBuilder {
    const W: f32 = 100.0;
    const H: f32 = Self::W * GOLDEN_RATIO;
    const MARGIN: f32 = 25.0;

    pub fn save(&self) {
        let cards = self
            .deck
            .iter()
            .map(|physical_card| physical_card.card.clone())
            .collect_vec();
        let json = serde_json::to_string(&cards).unwrap();
        std::fs::write("deck.json", json).unwrap();
    }

    pub fn load() -> Self {
        let cards_in_deck: Vec<Card> = std::fs::read_to_string("deck.json")
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();
        Self {
            card_pool: Card::iter()
                .map(|card| PhysicalCard {
                    card,
                    transform: Default::default(),
                    target_transform: RectTransform {
                        w: Self::W,
                        h: Self::H,
                        offset: Vec2::splat(0.5),
                        ..Default::default()
                    },
                })
                .collect(),
            deck: cards_in_deck
                .into_iter()
                .map(|card| PhysicalCard {
                    card,
                    transform: Default::default(),
                    target_transform: RectTransform {
                        w: Self::W,
                        h: Self::H,
                        offset: Vec2::splat(0.5),
                        ..Default::default()
                    },
                })
                .collect(),
            holding: None,
        }
    }

    pub fn step(&mut self, dt: f32) {
        for (cards, x_start) in [
            (&mut self.card_pool, 0.0),
            (&mut self.deck, screen_width() / 2.0),
        ] {
            let mut y = Self::MARGIN + Self::H / 2.0;
            for row in cards.iter_mut().chunks(3).into_iter() {
                let mut x = x_start + Self::MARGIN + Self::W / 2.0;
                for card in row {
                    card.target_transform.x = x;
                    card.target_transform.y = y;
                    x += Self::W + Self::MARGIN;
                }
                y += Self::H + Self::MARGIN;
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            assert!(self.holding.is_none());
            self.holding = pop_where(&mut self.deck, |physical_card| {
                point_inside(mouse_screen_position(), &physical_card.transform)
            })
            .or(self
                .card_pool
                .iter()
                .find(|physical_card| {
                    point_inside(mouse_screen_position(), &physical_card.transform)
                })
                .cloned());
        }

        if let Some(holding) = &mut self.holding {
            let mouse_pos = mouse_screen_position();
            holding.target_transform.x = mouse_pos.x;
            holding.target_transform.y = mouse_pos.y;
        }

        for physical_card in self.deck.iter_mut().chain(self.card_pool.iter_mut()) {
            let scale = if point_inside(mouse_screen_position(), &physical_card.transform) {
                1.2
            } else {
                1.0
            };
            physical_card.target_transform.h = Self::H * scale;
            physical_card.target_transform.w = Self::W * scale;
            physical_card
                .transform
                .animate_towards(&physical_card.target_transform, 20.0 * dt);
        }

        if let Some(holding) = &mut self.holding {
            holding
                .transform
                .animate_towards(&holding.target_transform, 20.0 * dt);
        }

        if is_mouse_button_released(MouseButton::Left) {
            if let Some(holding) = &self.holding {
                if mouse_screen_position().x > screen_width() / 2.0 {
                    self.deck.push(holding.clone());
                }
            }
            self.holding = None;
        }
    }

    pub fn draw(&self, sprites: &Sprites, font: Option<&Font>) {
        for physical_card in self.card_pool.iter() {
            draw_card(
                &physical_card.card,
                &physical_card.transform,
                1.0,
                sprites,
                font,
            )
        }
        for physical_card in self.deck.iter() {
            draw_card(
                &physical_card.card,
                &physical_card.transform,
                1.0,
                sprites,
                font,
            )
        }
        if let Some(card) = &self.holding {
            draw_card(&card.card, &card.transform, 1.0, sprites, font)
        }
    }
}

pub struct ClientGameState {
    time: SystemTime,
    pub server_controlled_game_state: ServerControledGameState,
    pub client_network_state: ClientNetworkState,
    pub in_deck_builder: bool,
    pub selected_entity_id: Option<EntityId>,
    pub player_id: PlayerId,
    pub dt: f32,
    pub sprites: Sprites,
    pub font: Font,
    pub unit_spawnpoint_targets: Vec<UnitSpawnpointTarget>,
    pub deck_builder: DeckBuilder,
    pub physical_hand: PhysicalHand,
    pub hit_numbers: HitNumbers,
    pub show_debug_info: bool,
    // TODO: temp
    pub card_delta_angle: f32,
    pub relative_splay_radius: f32,
}

impl ClientGameState {
    pub async fn new() -> Self {
        let client_network_state = ClientNetworkState::new();
        Self {
            server_controlled_game_state: Default::default(),
            player_id: client_network_state.get_player_id(),
            client_network_state,
            time: SystemTime::now(),
            in_deck_builder: true,
            show_debug_info: false,
            card_delta_angle: 0.1,
            relative_splay_radius: 4.5,
            selected_entity_id: None,
            dt: 0.167,
            sprites: load_sprites().await,
            font: macroquad::text::load_ttf_font("assets\\fonts\\shaky-hand-some-comic.bold.ttf")
                .await
                .unwrap(),
            unit_spawnpoint_targets: Vec::new(),
            deck_builder: DeckBuilder::load(),
            physical_hand: PhysicalHand::default(),
            hit_numbers: HitNumbers::new(),
        }
    }
    pub fn has_player(&self) -> bool {
        self.server_controlled_game_state
            .dynamic_game_state
            .players
            .get(&self.player_id)
            .is_some()
    }
    pub fn get_player(&self) -> &ServerPlayer {
        self.server_controlled_game_state
            .dynamic_game_state
            .players
            .get(&self.player_id)
            .unwrap()
    }
    pub fn step(&mut self) {
        let old_time = self.time;
        self.time = SystemTime::now();
        self.dt = self.time.duration_since(old_time).unwrap().as_secs_f32();
    }

    pub fn update_server_controled_game_state_with_server_message(
        &mut self,
        server_message: ServerMessage,
    ) {
        if self.server_controlled_game_state.game_metadata.server_tick
            >= server_message.metadata.server_tick
        {
            return;
        }
        self.server_controlled_game_state.game_metadata = server_message.metadata;
        match server_message.data {
            ServerMessageData::StaticGameState(static_state) => {
                self.server_controlled_game_state.static_game_state = static_state;
            }
            ServerMessageData::DynamicGameState(dynamic_state) => {
                self.server_controlled_game_state.dynamic_game_state = dynamic_state;
                hand_sync(self);
            }
            ServerMessageData::SemiStaticGameState(semi_static_state) => {
                self.server_controlled_game_state.semi_static_game_state = semi_static_state;
            }
        }
    }
}
