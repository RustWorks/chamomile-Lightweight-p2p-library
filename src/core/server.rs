use async_std::{
    io::Result,
    prelude::*,
    sync::{Arc, Mutex, Receiver, Sender},
    task,
};
use futures::{select, FutureExt};
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::transports::{
    new_channel, start as transport_start, Endpoint, EndpointMessage, StreamMessage, TransportType,
};
use crate::{Config, Message};

use super::keys::KeyType;
use super::peer::Peer;
use super::peer_list::PeerList;
use super::session::{start as session_start, RemotePublic};

/// start server
pub async fn start(
    config: Config,
    out_send: Sender<Message>,
    mut self_recv: Receiver<Message>,
) -> Result<()> {
    // TODO load or init config.
    // load or generate keypair
    let key = KeyType::Ed25519.generate_kepair();
    let key = Arc::new(key);

    let peer_id = key.peer_id();
    let mut peer_list = PeerList::init(peer_id);
    let peer = Peer::new(
        key.peer_id(),
        config.addr,
        TransportType::from_str(&config.transport),
        true,
    );
    let peer = Arc::new(peer);

    let _transports: HashMap<u8, Sender<EndpointMessage>> = HashMap::new();

    let (send, mut recv) = new_channel();
    let transport_send = transport_start(peer.transport(), peer.addr(), send.clone())
        .await
        .expect("Transport binding failure!");

    println!("Debug: listening: {}", config.addr);
    println!("Debug: peer id: {}", peer_id.short_show());

    task::spawn(async move {
        //let m1 = Arc::new(Mutex::new(server));
        //let m2 = m1.clone();

        loop {
            select! {
                msg = recv.next().fuse() => match msg {
                    Some(message) => {
                        match message {
                            EndpointMessage::PreConnected(addr, receiver, sender, is_ok) => {
                                // check and start session
                                if peer_list.is_black_addr(&addr) {
                                    sender.send(StreamMessage::Close).await;
                                } else {
                                    session_start(
                                        addr,
                                        receiver,
                                        sender,
                                        send.clone(),
                                        out_send.clone(),
                                        key.clone(),
                                        peer.clone(),
                                        is_ok,
                                    )
                                }
                            }
                            EndpointMessage::Connected(peer_id, sender, remote_peer) => {
                                // check and save tmp and save outside
                                if peer_list.is_black_peer(&peer_id) {
                                    sender.send(StreamMessage::Close).await;
                                } else {
                                    peer_list.add_tmp_peer(peer_id, sender, remote_peer);
                                    out_send.send(Message::PeerJoin(peer_id)).await;
                                }
                            }
                            EndpointMessage::Close(peer_id) => {
                                peer_list.remove(&peer_id);
                                out_send.send(Message::PeerLeave(peer_id)).await;
                            }
                            _ => {}
                        }
                    },
                    None => break,
                },
                msg = self_recv.next().fuse() => match msg {
                    Some(message) => {
                        match message {
                            Message::Connect(addr) => {
                                transport_send
                                    .send(EndpointMessage::Connect(
                                        addr,
                                        RemotePublic(key.public().clone(), *peer.clone()).to_bytes()
                                    ))
                                    .await;
                            }
                            Message::DisConnect(addr) => {
                                transport_send.send(EndpointMessage::Disconnect(addr)).await;
                            }
                            Message::PeerJoinResult(peer_id, is_ok) => {
                                let sender = peer_list.get(&peer_id);
                                if sender.is_some() {
                                    let sender = sender.unwrap();
                                    if is_ok {
                                        sender.send(StreamMessage::Ok).await;
                                        peer_list.stabilize_tmp_peer(peer_id);
                                    } else {
                                        sender.send(StreamMessage::Close).await;
                                        peer_list.remove_tmp_peer(&peer_id);
                                    }
                                }
                            }
                            Message::Data(peer_id, data) => {
                                let sender = peer_list.get(&peer_id);
                                if sender.is_some() {
                                    let sender = sender.unwrap();
                                    sender.send(StreamMessage::Bytes(data)).await;
                                }
                            },
                            Message::PeerLeave(peer_id) => {
                                let sender = peer_list.get(&peer_id);
                                if sender.is_some() {
                                    let sender = sender.unwrap();
                                    sender.send(StreamMessage::Close).await;
                                    peer_list.remove_tmp_peer(&peer_id);
                                }
                            },
                            Message::PeerJoin(_peer_id) => {},  // TODO search peer and join
                        }
                    },
                    None => break,
                }
            }
        }
        drop(send);
    });

    Ok(())
}
