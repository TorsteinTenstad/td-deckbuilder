use crate::card::{Card, CardInstance};
use crate::vector::shuffle_vec;
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Hand {
    pub card_draw_counter: f32,
    pub energy_counter: f32,
    pub energy: i32,
    pub cards: Vec<CardInstance>,
    pub deck: Vec<CardInstance>,
    pub played: Vec<CardInstance>,
}

impl Hand {
    pub fn new() -> Self {
        let mut deck = Vec::new();
        for (quantity, card) in vec![
            (3, Card::BasicTower),
            (5, Card::BasicUnit),
            (3, Card::SpawnPointTest),
            (3, Card::BasicRanger),
        ] {
            for _ in 0..quantity {
                let id = rand::thread_rng().gen();
                deck.push(CardInstance {
                    id,
                    card: card.clone(),
                });
            }
        }
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
        let card = self.deck.pop().unwrap();
        self.cards.push(card.clone());
        Some(card)
    }

    pub fn step(&mut self, dt: f32) {
        self.card_draw_counter += dt / 12.0;
        self.energy_counter += dt / 8.0;

        if self.card_draw_counter >= 1.0 {
            self.draw();
            self.card_draw_counter = 0.0;
        }
        if self.energy_counter >= 1.0 {
            self.energy = (self.energy + 1).min(10);
            self.energy_counter = 0.0;
        }
    }

    pub fn try_play(&mut self, card_id: u64) -> Option<Card> {
        let find_result = self
            .cards
            .clone() // TODO: Ask Amund
            .into_iter()
            .find_position(|card_instance| card_instance.id == card_id);
        let Some((card_idx, card_instance)) = find_result.clone() else {
            return None;
        };
        if self.energy < card_instance.card.energy_cost() {
            return None;
        }
        self.played.push(card_instance.clone());
        self.energy -= card_instance.card.energy_cost();
        self.cards.remove(card_idx);
        Some(card_instance.card)
    }
}
