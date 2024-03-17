use crate::{
    card::Card,
    game_state::{
        DynamicGameState, GameMetadata, SemiStaticGameState, ServerControlledGameState,
        StaticGameState,
    },
    ids::{CardInstanceId, PlayerId},
    message_acknowledgement::AckUdpSocket,
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
    JoinGame(Vec<Card>),
    PlayCard(CardInstanceId, PlayTarget),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessageData {
    StaticGameState(StaticGameState),
    SemiStaticGameState(SemiStaticGameState),
    DynamicGameState(DynamicGameState),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMessage {
    pub metadata: GameMetadata,
    pub data: ServerMessageData,
}

pub fn send_static_game_state<RxMessageContent: Serialize + for<'de> Deserialize<'de>>(
    ack_udp_socket: &mut AckUdpSocket<ServerMessage, RxMessageContent>,
    server_controlled_game_state: &ServerControlledGameState,
    client_addr: &SocketAddr,
) {
    ack_udp_socket.send_to(
        ServerMessage {
            metadata: server_controlled_game_state.game_metadata.clone(),
            data: ServerMessageData::StaticGameState(
                server_controlled_game_state.static_game_state.clone(),
            ),
        },
        client_addr,
        true,
    );
}

pub fn send_semi_static_game_state<RxMessageContent: Serialize + for<'de> Deserialize<'de>>(
    ack_udp_socket: &mut AckUdpSocket<ServerMessage, RxMessageContent>,
    server_controlled_game_state: &ServerControlledGameState,
    client_addr: &SocketAddr,
) {
    if !server_controlled_game_state.semi_static_game_state.dirty {
        return;
    }
    ack_udp_socket.send_to(
        ServerMessage {
            metadata: server_controlled_game_state.game_metadata.clone(),
            data: ServerMessageData::SemiStaticGameState(
                server_controlled_game_state.semi_static_game_state.clone(),
            ),
        },
        client_addr,
        true,
    );
}

pub fn send_dynamic_game_state<RxMessageContent: Serialize + for<'de> Deserialize<'de>>(
    ack_udp_socket: &mut AckUdpSocket<ServerMessage, RxMessageContent>,
    server_controlled_game_state: &ServerControlledGameState,
    client_addr: &SocketAddr,
) {
    ack_udp_socket.send_to(
        ServerMessage {
            metadata: server_controlled_game_state.game_metadata.clone(),
            data: ServerMessageData::DynamicGameState(
                server_controlled_game_state.dynamic_game_state.clone(),
            ),
        },
        client_addr,
        false,
    );
}
