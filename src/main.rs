pub mod message_handler;
pub mod session_manager;

use clap::Parser;
use futures::{channel::mpsc, SinkExt, StreamExt};
use message_io::{
    network::{Endpoint, ResourceId, Transport},
    node::{self, NodeHandler, NodeListener, StoredNetEvent, StoredNodeEvent},
};
use odin_networking::enc_session::EncDecSession;
use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    addr: SocketAddr,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let gameserver = GameServer::new(cli.addr).unwrap();
    gameserver.run().await;
}

pub struct GameServer {
    node_handler: NodeHandler<GameServerSignals>,
    node_listener: NodeListener<GameServerSignals>,
}
impl GameServer {
    pub fn new<T: ToSocketAddrs>(addr: T) -> io::Result<Self> {
        let (node_handler, node_listener) = node::split::<GameServerSignals>();
        node_handler.network().listen(Transport::Tcp, addr)?;

        Ok(Self {
            node_handler,
            node_listener,
        })
    }

    pub async fn run(self) {
        let mut attributes = GameServerContext {
            handler: self.node_handler,
            clients: Default::default(),
        };

        let (mut tx, mut rx) = mpsc::unbounded::<StoredNodeEvent<GameServerSignals>>();
        let (_task, mut enqueue) = self.node_listener.enqueue();
        tokio::spawn(async move {
            while !tx.is_closed() {
                let event = enqueue.receive();
                let _ = tx.send(event).await;
            }
        });

        while let Some(event) = rx.next().await {
            match event {
                StoredNodeEvent::Network(network) => match network {
                    StoredNetEvent::Connected(_, _) => unreachable!(),
                    StoredNetEvent::Accepted(_endpoint, resource_id) => {
                        if let Some(_client) = attributes.clients.get(&resource_id) {
                            panic!("Accepted a duplicated connection, this is a bug");
                        }

                        // let client_id = match attributes.next_client_id() {
                        //     Some(client_id) => client_id,
                        //     None => {
                        //         attributes.handler.network().remove(endpoint.resource_id());
                        //         return;
                        //     }
                        // };

                        // let keytable = std::rc::Rc::new([0u8; 512]);
                        // attributes.clients.insert(
                        //     resource_id,
                        //     Session::new(endpoint, EncDecSession::new(client_id, keytable)),
                        // );

                        // log::info!("Users online {}", attributes.clients.len());
                    }
                    StoredNetEvent::Message(endpoint, _message) => {
                        let Some(_session) = attributes.clients.get(&endpoint.resource_id()) else {
                            log::error!("Received a message from unknown endpoint");
                            return;
                        };
                    }
                    StoredNetEvent::Disconnected(endpoint) => {
                        if attributes.clients.remove(&endpoint.resource_id()).is_none() {
                            log::error!(
                                "Received a disconnect event from a unknown resource: {:?}",
                                endpoint
                            );
                        }
                    }
                },
                StoredNodeEvent::Signal(signal) => {
                    attributes
                        .handler
                        .signals()
                        .send_with_timer(signal, signal.interval());
                }
            };
        }
    }
}

pub struct GameServerContext {
    handler: NodeHandler<GameServerSignals>,
    clients: HashMap<ResourceId, Session>,
}

#[derive(Debug)]
pub struct Session {
    endpoint: Endpoint,
    _encdec_session: EncDecSession,
}
impl Session {
    pub fn new(endpoint: Endpoint, encdec_session: EncDecSession) -> Self {
        Self {
            endpoint,
            _encdec_session: encdec_session,
        }
    }
}
impl std::hash::Hash for Session {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.endpoint.resource_id().hash(state);
    }
}
impl PartialEq for Session {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserSession {
    Logging,
    SelChar,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameServerSignals {
    MobMovement,
}
impl GameServerSignals {
    pub fn all() -> [GameServerSignals; 1] {
        [GameServerSignals::MobMovement]
    }

    pub fn interval(&self) -> Duration {
        match self {
            GameServerSignals::MobMovement => Duration::from_millis(500),
        }
    }
}
