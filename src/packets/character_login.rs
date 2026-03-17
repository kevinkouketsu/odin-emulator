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
            client_id: self.entity_id.id() as u16,
            name: self.name.clone(),
            class: self.class,
            evolution: self.evolution,
            merchant: self.merchant,
            guild: self.guild,
            guild_level: self.guild_level,
            affect_info: self.affect_info,
            quest_info: self.quest_info,
            coin: self.coin,
            experience: self.experience,
            last_pos: self.last_pos,
            equipments: self.equipments.clone(),
            inventory: self.inventory.clone(),
            base_score: self.score,
            current_score: self.computed.score,
            score_bonus: self.score_bonus,
            special_bonus: self.special_bonus,
            skill_bonus: self.skill_bonus,
            critical: self.computed.critical.raw(),
            save_mana: self.computed.save_mana,
            magic: self.computed.magic,
            regen_hp: self.computed.regen_hp,
            regen_mp: self.computed.regen_mp,
            resist: self.computed.resist,
        }
    }
}
