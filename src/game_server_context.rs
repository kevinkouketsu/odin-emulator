use crate::{
    client_id_manager::ClientIdManager,
    configuration::{Configuration, ServerState},
    handlers::authentication::CliVer,
    user_session::UserSession,
    GameServerSignals,
};
use message_io::{network::ResourceId, node::NodeHandler};
use odin_repositories::account_repository::AccountRepository;
use std::{collections::HashMap, rc::Rc};
use tokio::sync::RwLock;

pub struct GameServerContext<A: AccountRepository> {
    handler: NodeHandler<GameServerSignals>,
    // TODO: Improve this RwLock
    sessions: HashMap<usize, Rc<RwLock<UserSession>>>,
    client_id_manager: ClientIdManager,
    current_cliver: CliVer,
    // TODO: temporary
    pub account_repository: A,
}
impl<A> GameServerContext<A>
where
    A: AccountRepository,
{
    pub fn new(
        handler: NodeHandler<GameServerSignals>,
        client_id_manager: ClientIdManager,
        account_repository: A,
    ) -> Self {
        Self {
            handler,
            sessions: Default::default(),
            client_id_manager,
            current_cliver: CliVer::new(11022),
            account_repository,
        }
    }

    pub fn get_client_id_manager_mut(&mut self) -> &mut ClientIdManager {
        &mut self.client_id_manager
    }

    pub fn add_session(&mut self, client_id: usize, session: UserSession) {
        self.sessions
            .insert(client_id, Rc::new(RwLock::new(session)));
    }

    pub fn get_sessions(&self) -> &HashMap<usize, Rc<RwLock<UserSession>>> {
        &self.sessions
    }

    pub fn get_handler(&self) -> &NodeHandler<GameServerSignals> {
        &self.handler
    }

    pub async fn get_client_id_by_resource_id(&self, resource_id: ResourceId) -> Option<usize> {
        for session in self.sessions.iter() {
            match session.1.read().await.get_resource_id() == resource_id {
                true => return Some(*session.0),
                false => continue,
            }
        }

        None
    }

    pub fn get_session_mut_by_client_id(
        &mut self,
        client_id: usize,
    ) -> Option<Rc<RwLock<UserSession>>> {
        self.sessions.get(&client_id).cloned()
    }
}
impl<A> Configuration for GameServerContext<A>
where
    A: AccountRepository,
{
    fn get_current_cliver(&self) -> CliVer {
        self.current_cliver
    }

    fn get_server_state(&self) -> ServerState {
        ServerState::Maintenance
    }
}
