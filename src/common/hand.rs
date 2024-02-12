use crate::card::{Card, CardInstance};
use crate::ids::CardInstanceId;
use crate::vector::{pop_where, shuffle_vec};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hand {
    pub card_draw_counter: f32,
    pub energy_counter: f32,
    pub energy: i32,
    pub cards: Vec<CardInstance>,
    pub deck: Vec<CardInstance>,
    pub played: Vec<CardInstance>,
}

impl Hand {
    pub fn new(deck: Vec<Card>) -> Self {
        let mut deck = deck
            .into_iter()
            .map(|card| CardInstance {
                id: CardInstanceId::new(),
                card,
            })
            .collect_vec();
        shuffle_vec(&mut deck);
        Self {
            card_draw_counter: 0.0,
            energy_counter: 0.0,
            energy: 0,
            cards: Vec::new(),
            deck,
            played: Vec::new(),
        }
    }

    pub fn draw(&mut self) -> Option<CardInstance> {
        if self.cards.len() >= 10 {
            return None;
        }
        if self.deck.is_empty() {
            self.deck = self.played.clone();
            self.played.clear();
            shuffle_vec(&mut self.deck);
        }
        let card = self.deck.pop()?; // TODO: How to handle all cards drawn? Currently, we don't draw.
        self.cards.push(card.clone());
        Some(card)
    }

    pub fn step(&mut self, dt: f32) {
        self.card_draw_counter += dt / 30.0;
        self.energy_counter += dt / 10.0;

        if self.card_draw_counter >= 1.0 {
            self.draw();
            self.card_draw_counter = 0.0;
        }
        if self.energy_counter >= 1.0 {
            self.energy = (self.energy + 1).min(10);
            self.energy_counter = 0.0;
        }
    }

    pub fn try_play(&mut self, card_id: CardInstanceId) -> Option<Card> {
        let Some(card_instance) = pop_where(&mut self.cards, |card_instance| {
            card_instance.id == card_id && card_instance.card.energy_cost() <= self.energy
        }) else {
            return None;
        };
        self.played.push(card_instance.clone());
        self.energy -= card_instance.card.energy_cost();
        Some(card_instance.card)
    }

    pub fn try_play_(&mut self, card_id: CardInstanceId) -> Option<Card> {
        let card_instance = pop_where(&mut self.cards, |card_instance| {
            card_instance.id == card_id && card_instance.card.energy_cost() <= self.energy
        })?;
        self.played.push(card_instance.clone());
        self.energy -= card_instance.card.energy_cost();
        Some(card_instance.card)
    }
}
