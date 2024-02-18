use crate::{
    deck_builder::DeckBuilder,
    draw::Sprites,
    hit_numbers::HitNumbers,
    physical_hand::{hand_sync, PhysicalHand},
    ClientNetworkState,
};
use common::{
    game_state::ServerControledGameState,
    ids::{EntityId, PlayerId},
    network::{ServerMessage, ServerMessageData},
    play_target::UnitSpawnpointTarget,
    server_player::ServerPlayer,
};
use macroquad::text::Font;
use std::time::SystemTime;

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
        let sprites = Sprites::load().await;
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
            sprites,
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
            > server_message.metadata.server_tick
            && server_message.metadata.game_id
                == self.server_controlled_game_state.game_metadata.game_id
        {
            åreturn;
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
