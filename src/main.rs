pub mod client_id_manager;
pub mod configuration;
pub mod game_server_context;
pub mod handlers;
pub mod map;
pub mod message;
pub mod packets;
pub mod score;
pub mod services;
pub mod session;
pub mod user_session;
pub mod world;

use bytes::Bytes;
use clap::Parser;
use client_id_manager::ClientIdManager;
use deku::prelude::*;
use game_server_context::GameServerContext;
use map::EntityId;
use message::{Message, MessageError};
use odin_database::DatabaseService;
use odin_models::item_data::ItemDatabase;
use odin_networking::{
    enc_session::EncDecSession, framed_message::HandshakeState, messages::header::Header,
};
use std::{net::SocketAddr, rc::Rc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};
use user_session::{SenderSession, UserSession};
use world::World;

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

pub enum GameEvent {
    Connected {
        client_id: usize,
        writer: mpsc::UnboundedSender<Bytes>,
    },
    Message {
        client_id: usize,
        data: Vec<u8>,
    },
    Disconnected {
        client_id: usize,
    },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    addr: SocketAddr,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    env_logger::init();
    dotenvy::dotenv().unwrap();

    let database_url = dotenvy::var("DATABASE_URL").expect("Database URL is mandatory");

    let connection = DatabaseService::new(&database_url).await.unwrap();
    let account_repository = connection.account_repository();
    let mut context =
        GameServerContext::new(ClientIdManager::with_maximum(750), account_repository);
    let item_db = match std::fs::read("ItemList.csv") {
        Ok(bytes) => {
            let contents: String = bytes.iter().map(|&b| b as char).collect();
            let db = ItemDatabase::from_csv(&contents).expect("Failed to parse ItemList.csv");
            log::info!("Loaded ItemList.csv");
            db
        }
        Err(e) => {
            log::warn!("ItemList.csv not found: {e}, using empty item database");
            ItemDatabase::default()
        }
    };
    let mut world = World::new(item_db);

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<GameEvent>();
    let listener = TcpListener::bind(cli.addr).await.unwrap();
    log::info!("Listening on {}", cli.addr);

    let keytable = Rc::new(KEYTABLE);

    loop {
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                let client_id = match context.allocate_client_id() {
                    Some(id) => id,
                    None => {
                        log::error!("Could not find a client id");
                        continue;
                    }
                };

                let (writer_tx, mut writer_rx) = mpsc::unbounded_channel::<Bytes>();
                let event_tx_clone = event_tx.clone();
                let (mut read_half, mut write_half) = stream.into_split();

                tokio::spawn(async move {
                    while let Some(data) = writer_rx.recv().await {
                        if write_half.write_all(&data).await.is_err() {
                            break;
                        }
                    }
                });

                tokio::spawn(async move {
                    let mut handshake = HandshakeState::default();
                    let mut buf = [0u8; 4096];

                    loop {
                        match read_half.read(&mut buf).await {
                            Ok(0) | Err(_) => {
                                let _ = event_tx_clone.send(GameEvent::Disconnected { client_id });
                                break;
                            }
                            Ok(n) => {
                                handshake.update(&buf[..n]);
                                while let Some(msg) = handshake.next_message() {
                                    if event_tx_clone.send(GameEvent::Message { client_id, data: msg }).is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                });

                let encdec = EncDecSession::new(client_id as u16, keytable.clone());
                context.add_sender(
                    client_id,
                    SenderSession::new(encdec.clone(), writer_tx.clone()),
                );
                context.add_session(
                    client_id,
                    UserSession::new(client_id, writer_tx.clone(), encdec),
                );

                log::info!("Player {} connected. ClientId: {}", addr, client_id);
            }

            Some(event) = event_rx.recv() => {
                match event {
                    GameEvent::Connected { .. } => {}
                    GameEvent::Message { client_id, mut data } => {
                        let Some(mut session) = context.take_session(client_id) else {
                            log::error!("Received a message from unknown client {}", client_id);
                            continue;
                        };

                        if let Err(e) = session.decrypt(&mut data) {
                            log::error!("Fail to decrypt packet: {:?}", e);
                            context.add_session(client_id, session);
                            continue;
                        }

                        let (rest, header) = Header::from_bytes((&data, 0))
                            .expect("Could not parse header this is very strange");

                        let message = match Message::try_from((rest, header)) {
                            Ok(message) => message,
                            Err(MessageError::NotImplemented(header)) => {
                                log::error!(
                                    "Received a packet that is not implemented yet: {:?}",
                                    header
                                );
                                context.add_session(client_id, session);
                                continue;
                            }
                            Err(MessageError::NotRecognized(header)) => {
                                log::error!(
                                    "Received a packet that has not been identified: {:?}",
                                    header
                                );
                                context.add_session(client_id, session);
                                continue;
                            }
                            Err(err) => {
                                log::error!("Invalid packet received: {:?}", err);
                                context.add_session(client_id, session);
                                continue;
                            }
                        };

                        log::info!("Received packet {:?} from {}", message, client_id);
                        session.handle(&context, &mut world, message).await;
                        context.add_session(client_id, session);
                    }
                    GameEvent::Disconnected { client_id } => {
                        if let Ok(_result) = world.remove_entity(EntityId::Player(client_id)) {
                            // TODO: Send remove packets to _result.spectators via context.send_to()
                        }
                        if context.disconnect(client_id).is_err() {
                            log::error!(
                                "Received a disconnect event from unknown ClientId: {}",
                                client_id
                            );
                        } else {
                            log::info!("Player disconnected. ClientId: {}", client_id);
                        }
                    }
                }
            }
        }
    }
}
