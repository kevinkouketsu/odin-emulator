use crate::handlers::authentication::CliVer;

pub trait Configuration {
    fn get_current_cliver(&self) -> CliVer;
    fn get_server_state(&self) -> ServerState;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Open,
    #[default]
    Maintenance,
}
