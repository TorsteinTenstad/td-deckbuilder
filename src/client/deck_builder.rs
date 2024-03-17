use common::{
    card::Card,
    config::CARD_ASPECT_RATIO,
    draw::{draw_card, Sprites},
    rect_transform::{point_inside, RectTransform},
    vector::pop_where,
};
use itertools::Itertools;
use macroquad::{
    input::{is_mouse_button_pressed, is_mouse_button_released},
    math::Vec2,
    miniquad::MouseButton,
    window::screen_width,
};

use crate::{input::mouse_screen_position, physical_card::PhysicalCard};

pub struct DeckBuilder {
    pub card_pool: Vec<PhysicalCard>,
    pub deck: Vec<PhysicalCard>,
    pub holding: Option<PhysicalCard>,
}

impl DeckBuilder {
    const W: f32 = 150.0;
    const H: f32 = Self::W * CARD_ASPECT_RATIO;
    const MARGIN: f32 = 5.0;

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
            for row in cards.iter_mut().chunks(4).into_iter() {
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

    pub fn draw(&self, sprites: &Sprites) {
        for physical_card in self.card_pool.iter() {
            draw_card(&physical_card.card, &physical_card.transform, 1.0, sprites)
        }
        for physical_card in self.deck.iter() {
            draw_card(&physical_card.card, &physical_card.transform, 1.0, sprites)
        }
        if let Some(card) = &self.holding {
            draw_card(&card.card, &card.transform, 1.0, sprites)
        }
    }
}
