use crate::{
    client_id_manager::{ClientIdManager, ClientIdManagerError},
    configuration::{CliVer, Configuration, ServerState},
    user_session::UserSession,
};
use odin_repositories::account_repository::AccountRepository;
use std::collections::HashMap;

pub struct GameServerContext<A: AccountRepository> {
    sessions: HashMap<usize, UserSession>,
    client_id_manager: ClientIdManager,
    current_cliver: CliVer,
    pub account_repository: A,
}
impl<A> GameServerContext<A>
where
    A: AccountRepository,
{
    pub fn new(client_id_manager: ClientIdManager, account_repository: A) -> Self {
        Self {
            sessions: Default::default(),
            client_id_manager,
            current_cliver: CliVer::new(11022),
            account_repository,
        }
    }

    pub fn allocate_client_id(&mut self) -> Option<usize> {
        self.client_id_manager.add()
    }

    pub fn release_client_id(&mut self, client_id: usize) -> Result<(), ClientIdManagerError> {
        self.client_id_manager.remove(client_id)
    }

    pub fn add_session(&mut self, client_id: usize, session: UserSession) {
        self.sessions.insert(client_id, session);
    }

    pub fn take_session(&mut self, client_id: usize) -> Option<UserSession> {
        self.sessions.remove(&client_id)
    }

    pub fn disconnect(&mut self, client_id: usize) -> Result<(), ClientIdManagerError> {
        self.sessions.remove(&client_id);
        self.client_id_manager.remove(client_id)
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
