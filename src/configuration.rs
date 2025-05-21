pub trait Configuration {
    fn get_current_cliver(&self) -> CliVer;
    fn get_server_state(&self) -> ServerState;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CliVer(u32);
impl CliVer {
    pub fn new(cliver: u32) -> Self {
        CliVer(cliver)
    }

    pub fn from_encrypted(cliver: u32) -> Self {
        CliVer(cliver.wrapping_shr((cliver & 28).wrapping_shr(2).wrapping_add(5)))
    }

    pub fn get_version(&self) -> u32 {
        self.0
    }
}
impl From<CliVer> for u32 {
    fn from(value: CliVer) -> Self {
        value.0
    }
}
impl PartialEq<u32> for CliVer {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Open,
    #[default]
    Maintenance,
}
