use common::{card::Card, play_target::WorldPosTarget, ClientCommand, PlayTarget};
use macroquad::{
    input::{is_mouse_button_pressed, is_mouse_button_released},
    math::Vec2,
    miniquad::MouseButton,
};
use rand::Rng;

use crate::{
    draw::{
        card_is_hovering, card_transform, hovered_card_transform, out_of_hand_card_transform,
        RectTransform,
    },
    input::{mouse_position_vec, mouse_world_position},
    ClientGameState,
};

fn shuffle_vec<T>(vec: &mut Vec<T>) {
    let mut rng = rand::thread_rng();
    for i in 0..vec.len() {
        let j = rng.gen_range(0..vec.len());
        vec.swap(i, j);
    }
}

pub struct PhysicalCard {
    pub card: Card,
    pub transform: RectTransform,
    pub target_transform: RectTransform,
}

pub struct Hand {
    pub card_draw_counter: i32,
    pub energy_counter: i32,
    pub energy: i32,
    pub hand: Vec<PhysicalCard>,
    pub card_idx_being_held: Option<usize>,
    pub deck: Vec<Card>,
    pub played: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        let mut deck = Vec::new();
        for (quantity, card) in vec![
            (3, Card::BasicTower),
            (5, Card::BasicUnit),
            (3, Card::BasicRanger),
        ] {
            for _ in 0..quantity {
                deck.push(card.clone());
            }
        }
        shuffle_vec(&mut deck);
        Self {
            card_draw_counter: 0,
            energy_counter: 0,
            energy: 0,
            hand: Vec::new(),
            card_idx_being_held: None,
            deck,
            played: Vec::new(),
        }
    }

    pub fn sync_with_server_counters(
        &mut self,
        server_card_draw_counter: i32,
        server_energy_counter: i32,
    ) {
        while self.card_draw_counter < server_card_draw_counter {
            self.draw();
            self.card_draw_counter += 1;
        }
        while self.energy_counter < server_energy_counter {
            self.energy = (self.energy + 1).min(10);
            self.energy_counter += 1;
        }
    }

    pub fn draw(&mut self) -> Option<Card> {
        if self.hand.len() >= 10 {
            return None;
        }
        if self.deck.is_empty() {
            self.deck = self.played.clone();
            self.played.clear();
            shuffle_vec(&mut self.deck);
        }
        let card = self.deck.pop().unwrap();
        self.hand.push(PhysicalCard {
            card: card.clone(),
            transform: Default::default(),
            target_transform: Default::default(),
        });
        Some(card)
    }

    pub fn try_release_held_card(&mut self) -> Option<Card> {
        let Some(card_idx_being_held) = self.card_idx_being_held else {
            return None;
        };
        if self.energy
            < self
                .hand
                .get(card_idx_being_held)
                .unwrap()
                .card
                .energy_cost()
        {
            return None;
        }
        let card = self.hand.remove(card_idx_being_held).card;
        self.energy -= card.energy_cost();
        self.played.push(card.clone());
        Some(card)
    }
}

pub fn player_step(state: &mut ClientGameState) {
    let hand_size = state.hand.hand.len();
    let mut top_hovering_card_idx: Option<usize> = None;
    if let Some(card_idx_being_held) = state.hand.card_idx_being_held {
        let Vec2 { x, y } = mouse_position_vec();
        state
            .hand
            .hand
            .get_mut(card_idx_being_held)
            .unwrap()
            .target_transform = out_of_hand_card_transform(x, y);

        if is_mouse_button_released(MouseButton::Left) {
            if let Some(card) = state.hand.try_release_held_card() {
                let Vec2 { x, y } = mouse_world_position();
                state.commands.push(ClientCommand::PlayCard(
                    card,
                    PlayTarget::WorldPos(WorldPosTarget { x, y }),
                ));
            }
            state.hand.card_idx_being_held = None;
        }
    } else {
        for (i, physical_card) in state.hand.hand.iter_mut().enumerate() {
            let in_hand_transform = card_transform(
                i,
                hand_size,
                state.relative_splay_radius,
                state.card_delta_angle,
            );
            if card_is_hovering(&in_hand_transform) {
                top_hovering_card_idx = Some(i);
                if is_mouse_button_pressed(MouseButton::Left) {
                    state.hand.card_idx_being_held = Some(i);
                }
            }
            physical_card.target_transform = in_hand_transform;
        }
        if let Some(i) = top_hovering_card_idx {
            state.hand.hand.get_mut(i).unwrap().target_transform = hovered_card_transform(
                i,
                hand_size,
                state.relative_splay_radius,
                state.card_delta_angle,
            );
        }
    }

    for physical_card in state.hand.hand.iter_mut() {
        physical_card
            .transform
            .animate_towards(&physical_card.target_transform, state.dt * 20.0);
    }
}
