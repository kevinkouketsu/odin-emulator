use crate::world::Player;
use odin_models::position::Position;
use odin_networking::messages::server::character_login::CharacterLogin;

pub trait ToCharacterLogin {
    fn to_character_login(&self, position: Position, client_id: u16) -> CharacterLogin;
}

impl ToCharacterLogin for Player {
    fn to_character_login(&self, position: Position, client_id: u16) -> CharacterLogin {
        let base_character = self.base_character();
        CharacterLogin {
            position,
            character: base_character.clone(),
            client_id,
        }
    }
}
