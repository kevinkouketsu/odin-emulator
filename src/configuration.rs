use crate::handlers::authentication::CliVer;

pub trait Configuration {
    fn get_current_cliver(&self) -> CliVer;
}
