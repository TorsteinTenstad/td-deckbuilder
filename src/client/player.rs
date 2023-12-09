use common::card::Card;
use rand::Rng;

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
