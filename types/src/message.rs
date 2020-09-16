use std::net::SocketAddr;

use crate::types::{Broadcast, PeerId, TransportStream, TransportType};

/// Custom apply for build a stream between nodes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StreamType {
    /// request for build a stream, params is peer id, transport type and request custom info.
    Req(PeerId, TransportType, Vec<u8>),
    /// response for build a stream, params is is_ok, and response custom info.
    Res(bool, Vec<u8>),
    /// if response is ok, will build a stream, and return the stream to ouside.
    Ok(TransportStream),
}

/// main received message for outside channel, send from chamomile to outside.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReceiveMessage {
    /// (permissioned only) when a stable peer join, send from chamomile to outside.
    /// params is `peer_id`, `socket_addr` and peer `join_info`.
    PeerJoin(PeerId, SocketAddr, Vec<u8>),
    /// (permissioned only) when peer get join result.
    /// params is `peer_id`, `is_ok` and `result_data`.
    PeerJoinResult(PeerId, bool, Vec<u8>),
    /// when a stable connection's peer leave, send from chamomile to outside.
    /// params is `peer_id`.
    PeerLeave(PeerId),
    /// when received a data from a trusted peer, send to outside.
    /// params is `peer_id` and `data_bytes`.
    Data(PeerId, Vec<u8>),
    /// Apply for build a stream between nodes.
    /// params is `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType),
}

/// main send message for outside channel, send from outside to chamomile.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SendMessage {
    /// (permissioned only) when peer request for join, outside decide connect or not.
    /// params is `peer_id`, `is_connect`, `is_force_close`, `result info`.
    /// if `is_connect` is true, it will add to white directly list.
    /// we want to build a better network, add a `is_force_close`.
    /// if `is_connect` is false, but `is_force_close` if true, we
    /// will use this peer to build our DHT for better connection.
    /// if false, we will force close it.
    PeerJoinResult(PeerId, bool, bool, Vec<u8>),
    /// when need add a peer to stable connect, send to chamomile from outside.
    /// if success connect, will start a stable connection, and add peer to kad, stables,
    /// bootstraps and whitelists. if failure, will send `PeerLeave` to outside.
    /// params is `peer_id`, `socket_addr` and peer `join_info`.
    PeerConnect(PeerId, Option<SocketAddr>, Vec<u8>),
    /// when outside want to close a stable connectioned peer. use it force close.
    /// params is `peer_id`.
    PeerDisconnect(PeerId),
    /// when outside want to connect a peer. will try connect directly.
    /// if connected, chamomile will add to kad and bootstrap. if join_info is none,
    /// chamomile will use config's join_data as default.
    /// params is `socket_addr`, `join_info`.
    Connect(SocketAddr, Option<Vec<u8>>),
    /// when outside donnot want to connect peer. use it to force close.
    /// it will remove from kad and bootstrap list.
    /// params is `socket_addr`.
    DisConnect(SocketAddr),
    /// when need send a data to a peer, only need know the peer_id,
    /// the chamomile will help you send data to there.
    /// params is `peer_id` and `data_bytes`.
    Data(PeerId, Vec<u8>),
    /// when need broadcast a data to all network, chamomile support some
    /// common algorithm, use it, donnot worry.
    /// params is `broadcast_type` and `data_bytes`
    Broadcast(Broadcast, Vec<u8>),
    /// Apply for build a stream between nodes.
    /// params is `u32` stream symbol, and `StreamType`.
    Stream(u32, StreamType),
}