use crate::{
    deck_builder::DeckBuilder, hit_numbers::HitNumbers, network::ClientNetworkState,
    physical_hand::PhysicalHand,
};
use common::{
    debug_draw_config::DebugDrawConfig,
    draw::Sprites,
    game_state::ServerControlledGameState,
    ids::{EntityId, PlayerId},
    server_player::ServerPlayer,
};
use macroquad::text::Font;
use std::time::SystemTime;

pub struct ClientGameState {
    time: SystemTime,
    pub server_controlled_game_state: ServerControlledGameState,
    pub client_network_state: ClientNetworkState,
    pub in_deck_builder: bool,
    pub selected_entity_id: Option<EntityId>,
    pub player_id: PlayerId,
    pub dt: f32,
    pub sprites: Sprites,
    pub font: Font,
    pub deck_builder: DeckBuilder,
    pub physical_hand: PhysicalHand,
    pub hit_numbers: HitNumbers,
    pub debug_draw_config: DebugDrawConfig,
    // TODO: temp
    pub card_delta_angle: f32,
    pub relative_splay_radius: f32,
}

impl ClientGameState {
    pub async fn new() -> Self {
        let client_network_state = ClientNetworkState::new();
        let sprites = Sprites::load().await;
        Self {
            server_controlled_game_state: Default::default(),
            player_id: client_network_state.get_player_id(),
            client_network_state,
            time: SystemTime::now(),
            in_deck_builder: true,
            debug_draw_config: DebugDrawConfig::default(),
            card_delta_angle: 0.1,
            relative_splay_radius: 4.5,
            selected_entity_id: None,
            dt: 0.167,
            sprites,
            font: macroquad::text::load_ttf_font("assets\\fonts\\shaky-hand-some-comic.bold.ttf")
                .await
                .unwrap(),
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
}
