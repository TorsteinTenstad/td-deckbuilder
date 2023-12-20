use crate::{
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
pub enum ClientCommand {
    PlayCard(CardInstanceId, PlayTarget),
    JoinGame,
}
