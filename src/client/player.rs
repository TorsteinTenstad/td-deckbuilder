use common::{card::Card, ClientCommand};
use macroquad::{
    input::{is_mouse_button_down, is_mouse_button_released},
    miniquad::MouseButton,
};
use rand::Rng;

use crate::{
    draw::{
        card_is_hovering, card_transform, draw_card, draw_highlighted_card, draw_out_of_hand_card,
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

pub struct Hand {
    pub card_draw_counter: i32,
    pub energy_counter: i32,
    pub energy: i32,
    pub hand: Vec<Card>,
    pub deck: Vec<Card>,
    pub played: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        let mut deck = Vec::new();
        for (quantity, card) in vec![
            (3, Card::BasicTower),
            (5, Card::BasicUnit),
            (3, Card::BasicDrone),
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
        self.hand.push(card.clone());
        Some(card)
    }

    pub fn try_move_card_from_hand_to_played(&mut self, index: usize) -> Option<Card> {
        if self.energy < self.hand.get(index).unwrap().energy_cost() {
            return None;
        }
        let card = self.hand.remove(index);
        self.energy -= card.energy_cost();
        self.played.push(card.clone());
        Some(card)
    }
}

pub fn player_step(state: &mut ClientGameState) {
    let input = &state.input;

    let highlighted_card_opt_clone = state.highlighted_card_opt.clone();
    state.highlighted_card_opt = None;

    for (i, card) in state.hand.hand.iter().enumerate() {
        let is_selected = highlighted_card_opt_clone == Some(i);

        let transform = card_transform(
            i,
            state.hand.hand.len(),
            state.relative_splay_radius,
            state.card_delta_angle,
        );
        let hovering = card_is_hovering(&transform);
        if !(is_selected && !is_mouse_button_down(MouseButton::Left)) {
            draw_card(card, &transform, 1.0, &state.textures);
        }
        if hovering {
            state.highlighted_card_opt = Some(i);
        }
    }

    let mouse_pos = mouse_position_vec();
    let mouse_world_pos = mouse_world_position();

    if let Some(highlighted_card) = highlighted_card_opt_clone {
        let card = state.hand.hand.get(highlighted_card).unwrap();

        if is_mouse_button_released(MouseButton::Left) {
            if input.mouse_in_world
                && match card {
                    Card::BasicTower => !input.mouse_over_occupied_tile,
                    Card::BasicRanger | Card::BasicDrone | Card::BasicUnit => true,
                }
            {
                if let Some(card) = state
                    .hand
                    .try_move_card_from_hand_to_played(highlighted_card)
                {
                    state.commands.push(ClientCommand::PlayCard(
                        mouse_world_pos.x,
                        mouse_world_pos.y,
                        card.clone(),
                    ));
                }
            }
            state.preview_tower_pos = None;
        } else {
            if is_mouse_button_down(MouseButton::Left) {
                state.highlighted_card_opt = Some(highlighted_card);
                if input.mouse_in_world {
                    match card {
                        Card::BasicTower => {
                            state.preview_tower_pos = Some((
                                mouse_world_pos.x as i32 as f32 + 0.5,
                                mouse_world_pos.y as i32 as f32 + 0.5,
                            ));
                        }
                        Card::BasicRanger | Card::BasicDrone | Card::BasicUnit => {
                            draw_out_of_hand_card(card, mouse_pos.x, mouse_pos.y, &state.textures);
                        }
                    }
                } else {
                    draw_out_of_hand_card(card, mouse_pos.x, mouse_pos.y, &state.textures);
                }
            } else {
                let transform = card_transform(
                    highlighted_card,
                    state.hand.hand.len(),
                    state.relative_splay_radius,
                    state.card_delta_angle,
                );
                let hovering = card_is_hovering(&transform);
                draw_highlighted_card(
                    card,
                    highlighted_card,
                    state.relative_splay_radius,
                    state.card_delta_angle,
                    &state.textures,
                    state.hand.hand.len(),
                );
                if hovering {
                    state.highlighted_card_opt = Some(highlighted_card);
                }
            }
        }
    }
}
