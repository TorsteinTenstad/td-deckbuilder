use crate::play_target::PlayTarget;
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    net::SocketAddr,
};

pub fn hash_client_addr(addr: &SocketAddr) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    addr.to_string().hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientCommand {
    PlayCard(u64, PlayTarget),
    JoinGame,
}
