use crate::{draw::load_sprites, network::udp_init_socket, physical_hand::PhysicalHand};
use common::{
    game_state::{DynamicGameState, StaticGameState},
    network::ClientCommand,
    play_target::UnitSpawnpointTarget,
    server_player::ServerPlayer,
    textures::SpriteId,
};
use macroquad::texture::Texture2D;
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

pub struct ClientGameState {
    time: SystemTime,
    pub static_game_state: StaticGameState,
    pub dynamic_game_state: DynamicGameState,
    pub selected_entity_id: Option<u64>,
    pub frames_since_last_received: i32,
    pub commands: Vec<ClientCommand>,
    pub udp_socket: UdpSocket,
    pub player_id: u64,
    pub dt: f32,
    pub sprites: HashMap<SpriteId, Texture2D>,
    pub unit_spawnpoint_targets: Vec<UnitSpawnpointTarget>,
    pub physical_hand: PhysicalHand,
    // TODO: temp
    pub card_delta_angle: f32,
    pub relative_splay_radius: f32,
}

impl ClientGameState {
    pub async fn new() -> Self {
        let (udp_socket, player_id) = udp_init_socket();

        Self {
            static_game_state: StaticGameState::new(),
            dynamic_game_state: DynamicGameState::new(),
            time: SystemTime::now(),
            card_delta_angle: 0.1,
            relative_splay_radius: 4.5,
            commands: Vec::new(),
            frames_since_last_received: 0,
            selected_entity_id: None,
            udp_socket,
            player_id,
            dt: 0.167,
            sprites: load_sprites().await,
            unit_spawnpoint_targets: Vec::new(),
            physical_hand: PhysicalHand::default(),
        }
    }
    pub fn get_player(&self) -> &ServerPlayer {
        self.dynamic_game_state
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
