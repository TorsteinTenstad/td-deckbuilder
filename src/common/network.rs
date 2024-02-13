use crate::{
    card::Card,
    game_state::{DynamicGameState, GameMetadata, SemiStaticGameState, StaticGameState},
    ids::{CardInstanceId, PlayerId, SemiStaticGameStateVersionId},
    play_target::PlayTarget,
};
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    net::SocketAddr,
};

pub fn hash_client_addr(addr: &SocketAddr) -> PlayerId {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    addr.to_string().hash(&mut hasher);
    let id = hasher.finish();
    PlayerId(id)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientPing {
    pub static_game_state_reseived: bool,
    pub semi_static_game_state_version_id: SemiStaticGameStateVersionId,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    JoinGame(Vec<Card>),
    Ping(ClientPing),
    PlayCard(CardInstanceId, PlayTarget),
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessageData {
    StaticGameState(StaticGameState),
    SemiStaticGameState(SemiStaticGameState),
    DynamicGameState(DynamicGameState),
}

#[derive(Serialize, Deserialize)]
pub struct ServerMessage {
    pub metadata: GameMetadata,
    pub data: ServerMessageData,
}
