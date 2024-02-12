use crate::{
    card::Card,
    game_state::{DynamicGameState, GameMetadata, StaticGameState},
    ids::{CardInstanceId, PlayerId},
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
pub enum ClientMessage {
    PlayCard(CardInstanceId, PlayTarget),
    JoinGame(Vec<Card>),
    RequestStaticGameState,
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessageData {
    StaticGameState(StaticGameState),
    DynamicGameState(DynamicGameState),
}

#[derive(Serialize, Deserialize)]
pub struct ServerMessage {
    pub metadata: GameMetadata,
    pub data: ServerMessageData,
}
