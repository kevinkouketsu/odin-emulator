pub mod client_id_manager;
pub mod configuration;
pub mod game_server_context;
pub mod handlers;
pub mod message;
pub mod session;
pub mod user_session;

use clap::Parser;
use client_id_manager::ClientIdManager;
use deku::prelude::*;
use futures::{channel::mpsc, SinkExt, StreamExt};
use game_server_context::GameServerContext;
use message::{Message, MessageError};
use message_io::{
    network::Transport,
    node::{self, NodeHandler, NodeListener, StoredNetEvent, StoredNodeEvent},
};
use odin_networking::{enc_session::EncDecSession, messages::header::Header};
use odin_postgresql::PostgresqlService;
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};
use user_session::UserSession;

const KEYTABLE: [u8; 512] = [
    0x14, 0x17, 0x47, 0x67, 0x7A, 0x09, 0x21, 0x0D, 0x5B, 0x5B, 0x15, 0x0D, 0x17, 0x11, 0x21, 0x0C,
    0x1F, 0x03, 0x21, 0x21, 0x17, 0x0D, 0x1D, 0x0D, 0x16, 0x1F, 0x03, 0x1F, 0x71, 0x6D, 0x15, 0x0D,
    0x15, 0x0D, 0x15, 0x13, 0x17, 0x2C, 0x15, 0x43, 0x1D, 0x72, 0x17, 0x29, 0x1F, 0x09, 0x15, 0x16,
    0x47, 0x0D, 0x67, 0x6D, 0x79, 0x0D, 0x67, 0x0D, 0x15, 0x09, 0x15, 0x0D, 0x1F, 0x71, 0x17, 0x0E,
    0x33, 0x17, 0x05, 0x09, 0x6F, 0x73, 0x5B, 0x13, 0x33, 0x32, 0x3E, 0x1E, 0x24, 0x0D, 0x6E, 0x0E,
    0x15, 0x0A, 0x15, 0x3F, 0x5D, 0x0D, 0x17, 0x35, 0x17, 0x0D, 0x71, 0x0D, 0x18, 0x0D, 0x25, 0x21,
    0x33, 0x0D, 0x17, 0x0C, 0x1D, 0x0A, 0x15, 0x17, 0x27, 0x0C, 0x15, 0x0D, 0x3C, 0x10, 0x4B, 0x09,
    0x14, 0x2B, 0x6B, 0x35, 0x67, 0x1F, 0x15, 0x1F, 0x15, 0x0E, 0x15, 0x10, 0x15, 0x28, 0x05, 0x2D,
    0x33, 0x2A, 0x1D, 0x29, 0x17, 0x0C, 0x15, 0x0D, 0x14, 0x0D, 0x15, 0x0E, 0x77, 0x27, 0x1D, 0x1F,
    0x15, 0x0B, 0x7A, 0x0D, 0x3D, 0x10, 0x3D, 0x0D, 0x47, 0x3F, 0x1D, 0x0D, 0x79, 0x4D, 0x15, 0x0D,
    0x17, 0x47, 0x33, 0x0D, 0x77, 0x47, 0x33, 0x1C, 0x17, 0x0E, 0x15, 0x35, 0x0D, 0x06, 0x45, 0x49,
    0x1D, 0x7F, 0x33, 0x0D, 0x17, 0x2B, 0x15, 0x1C, 0x71, 0x31, 0x1D, 0x0F, 0x17, 0x0D, 0x14, 0x0A,
    0x14, 0x0B, 0x71, 0x16, 0x78, 0x7F, 0x61, 0x09, 0x15, 0x29, 0x63, 0x25, 0x53, 0x57, 0x29, 0x0D,
    0x77, 0x1C, 0x47, 0x0C, 0x33, 0x0D, 0x15, 0x0D, 0x5B, 0x09, 0x31, 0x35, 0x17, 0x0D, 0x29, 0x0D,
    0x1D, 0x0D, 0x25, 0x21, 0x33, 0x0D, 0x17, 0x0C, 0x15, 0x0A, 0x15, 0x3F, 0x5D, 0x0D, 0x17, 0x0D,
    0x79, 0x4D, 0x15, 0x0D, 0x25, 0x09, 0x15, 0x0D, 0x51, 0x0B, 0x7A, 0x0D, 0x47, 0x0D, 0x15, 0x0D,
    0x15, 0x0D, 0x1D, 0x0D, 0x79, 0x03, 0x15, 0x09, 0x15, 0x0D, 0x67, 0x0D, 0x15, 0x71, 0x49, 0x71,
    0x1F, 0x75, 0x15, 0x16, 0x3D, 0x0D, 0x67, 0x6D, 0x33, 0x1E, 0x76, 0x0D, 0x6E, 0x0E, 0x3E, 0x1E,
    0x1F, 0x71, 0x19, 0x0E, 0x33, 0x0D, 0x05, 0x09, 0x33, 0x71, 0x5B, 0x13, 0x1C, 0x1F, 0x15, 0x0B,
    0x15, 0x0E, 0x1F, 0x10, 0x15, 0x28, 0x05, 0x0A, 0x15, 0x2A, 0x1D, 0x71, 0x1F, 0x0C, 0x19, 0x1C,
    0x15, 0x1B, 0x33, 0x79, 0x17, 0x0B, 0x33, 0x1C, 0x2F, 0x47, 0x31, 0x0A, 0x18, 0x0E, 0x1F, 0x35,
    0x0D, 0x10, 0x47, 0x49, 0x28, 0x4F, 0x5B, 0x29, 0x15, 0x35, 0x21, 0x10, 0x17, 0x11, 0x17, 0x0C,
    0x1F, 0x03, 0x21, 0x21, 0x14, 0x17, 0x47, 0x67, 0x16, 0x09, 0x71, 0x6D, 0x15, 0x0A, 0x03, 0x2B,
    0x15, 0x0D, 0x1D, 0x13, 0x17, 0x2C, 0x15, 0x43, 0x17, 0x0D, 0x15, 0x1F, 0x17, 0x0D, 0x1D, 0x0D,
    0x06, 0x0E, 0x17, 0x0D, 0x18, 0x29, 0x19, 0x05, 0x61, 0x6D, 0x15, 0x0D, 0x1B, 0x53, 0x7A, 0x0A,
    0x67, 0x40, 0x1D, 0x0D, 0x17, 0x35, 0x17, 0x0C, 0x03, 0x0E, 0x0D, 0x16, 0x17, 0x33, 0x15, 0x20,
    0x67, 0x6F, 0x7D, 0x35, 0x71, 0x0A, 0x15, 0x33, 0x7A, 0x0E, 0x15, 0x28, 0x3D, 0x09, 0x16, 0x0D,
    0x15, 0x0D, 0x67, 0x0D, 0x71, 0x0A, 0x05, 0x0D, 0x15, 0x40, 0x3B, 0x47, 0x71, 0x0A, 0x17, 0x09,
    0x14, 0x0D, 0x03, 0x03, 0x17, 0x0D, 0x33, 0x0D, 0x79, 0x0D, 0x15, 0x0E, 0x12, 0x0D, 0x6D, 0x3D,
    0x17, 0x09, 0x77, 0x09, 0x3D, 0x0C, 0x33, 0x6A, 0x17, 0x1D, 0x1D, 0x0B, 0x77, 0x09, 0x2B, 0x0D,
    0x67, 0x1F, 0x15, 0x0D, 0x1D, 0x44, 0x1F, 0x0D, 0x3D, 0x17, 0x79, 0x0C, 0x15, 0x10, 0x15, 0x09,
    0x1A, 0x53, 0x77, 0x35, 0x78, 0x7B, 0x1D, 0x04, 0x20, 0x03, 0x43, 0x27, 0x1D, 0x47, 0x31, 0x29,
];

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    addr: SocketAddr,

    database_url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    env_logger::init();

    let gameserver = GameServer::new(cli.addr, cli.database_url).unwrap();
    gameserver.run().await;
}

pub struct GameServer {
    node_handler: NodeHandler<GameServerSignals>,
    node_listener: NodeListener<GameServerSignals>,
    database_url: String,
}
impl GameServer {
    pub fn new<T: ToSocketAddrs>(addr: T, database_url: String) -> io::Result<Self> {
        let (node_handler, node_listener) = node::split::<GameServerSignals>();
        node_handler.network().listen(Transport::Tcp, addr)?;

        Ok(Self {
            node_handler,
            node_listener,
            database_url,
        })
    }

    pub async fn run(self) {
        let account_repository = PostgresqlService::new(&self.database_url).await.unwrap();
        let mut context = GameServerContext::new(
            self.node_handler.clone(),
            ClientIdManager::with_maximum(750),
            account_repository,
        );

        let (mut tx, mut rx) = mpsc::unbounded::<StoredNodeEvent<GameServerSignals>>();
        let (_task, mut enqueue) = self.node_listener.enqueue();
        let _handle = tokio::spawn(async move {
            while !tx.is_closed() {
                let event = enqueue.receive();
                let _ = tx.send(event).await;
            }
        });

        while let Some(event) = rx.next().await {
            match event {
                StoredNodeEvent::Network(network) => match network {
                    StoredNetEvent::Connected(_, _) => unreachable!(),
                    StoredNetEvent::Accepted(endpoint, _resource_id) => {
                        let resource_id = endpoint.resource_id();
                        let client_id = match context.get_client_id_manager_mut().add() {
                            Some(client_id) => client_id,
                            None => {
                                context.get_handler().network().remove(resource_id);

                                log::error!("Could not find a client id");
                                return;
                            }
                        };

                        let keytable = std::rc::Rc::new(KEYTABLE);
                        context.add_session(
                            client_id,
                            UserSession::new(
                                self.node_handler.clone(),
                                endpoint,
                                EncDecSession::new(client_id as u16, keytable.clone()),
                            ),
                        );

                        log::info!(
                            "Player {:?} connected. ClientId: {}. Resource Id: {}",
                            endpoint,
                            client_id,
                            resource_id
                        );
                    }
                    StoredNetEvent::Message(endpoint, message) => {
                        let client_id = context
                            .get_client_id_by_resource_id(endpoint.resource_id())
                            .await
                            .expect("Could not find a clientid for a specific resource id");

                        let Some(session) = context.get_session_mut_by_client_id(client_id) else {
                            log::error!("Received a message from unknown endpoint");
                            return;
                        };

                        let mut session = session.write().await;
                        session.feed_with_message(&message);

                        while let Some(message) = session.next_message() {
                            let decrypted_message = match session.decrypt(&message) {
                                Ok(decrypted_message) => decrypted_message,
                                Err(e) => {
                                    log::error!("Fail to decrypt packet: {:?}", e);

                                    return;
                                }
                            };

                            let (_, header) = Header::from_bytes((&decrypted_message, 0))
                                .expect("Could not parse header this is very strange");

                            let message = match Message::try_from((
                                (
                                    &decrypted_message.as_slice()[std::mem::size_of::<Header>()..],
                                    0,
                                ),
                                header,
                            )) {
                                Ok(message) => message,
                                Err(MessageError::NotImplemented(header)) => {
                                    log::error!(
                                        "Received a packet that is not implemented yet: {:?}",
                                        header
                                    );

                                    return;
                                }
                                Err(MessageError::NotRecognized(header)) => {
                                    log::error!(
                                        "Received a packet that has not been identified: {:?}",
                                        header
                                    );

                                    return;
                                }
                                Err(err) => {
                                    log::error!("Invalid packet received: {:?}", err);
                                    return;
                                }
                            };

                            log::info!(
                                "{:?}",
                                message
                                    .handle(&session, &context, context.account_repository.clone())
                                    .await
                            );
                            log::info!("Received packet {:?} from {}", message, client_id);
                        }
                    }
                    StoredNetEvent::Disconnected(endpoint) => {
                        let client_id = context
                            .get_client_id_by_resource_id(endpoint.resource_id())
                            .await
                            .expect("Could not find a clientid for a specific resource id");

                        if context
                            .get_client_id_manager_mut()
                            .remove(client_id)
                            .is_err()
                        {
                            log::error!(
                                "Received a disconnect event from a unknown resource: {:?}. Resource Id: {}. ClientId: {}",
                                endpoint,
                                endpoint.resource_id(), client_id
                            );
                        }
                    }
                },
                StoredNodeEvent::Signal(_signal) => {}
            };
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameServerSignals {}
