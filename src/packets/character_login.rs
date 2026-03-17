use crate::world::Player;
use odin_models::position::Position;
use odin_networking::messages::server::character_login::CharacterLogin;

pub trait ToCharacterLogin {
    fn to_character_login(&self, position: Position) -> CharacterLogin;
}

impl ToCharacterLogin for Player {
    fn to_character_login(&self, position: Position) -> CharacterLogin {
        CharacterLogin {
            position,
            character: self.into(),
            client_id: self.entity_id.id() as u16,
        }
    }
}
