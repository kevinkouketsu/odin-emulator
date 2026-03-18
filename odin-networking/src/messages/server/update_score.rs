use crate::{
    WritableResource, WritableResourceError,
    messages::{ServerMessage, common::ScoreRaw},
};
use deku::prelude::*;
use odin_models::{MAX_AFFECT, status::Score};

pub struct UpdateScore {
    pub mob_id: u16,
    pub score: Score,
    pub critical: u8,
    pub save_mana: i8,
    pub affect: [u8; MAX_AFFECT],
    pub guild: u16,
    pub guild_level: u16,
    pub resist: [i8; 4],
    pub req_hp: i32,
    pub req_mp: i32,
    pub magic: i32,
    pub rsv: u16,
    pub learned_skill: i8,
}

impl WritableResource for UpdateScore {
    const IDENTIFIER: ServerMessage = ServerMessage::UpdateScore;
    type Output = UpdateScoreRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(UpdateScoreRaw {
            score: self.score.into(),
            critical: self.critical as i8,
            save_mana: self.save_mana,
            affect: self.affect,
            guild: self.guild,
            guild_level: self.guild_level,
            resist: self.resist,
            req_hp: self.req_hp,
            req_mp: self.req_mp,
            magic: self.magic,
            rsv: self.rsv,
            learned_skill: self.learned_skill,
        })
    }

    fn client_id(&self) -> Option<u16> {
        Some(self.mob_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite)]
pub struct UpdateScoreRaw {
    pub score: ScoreRaw,
    pub critical: i8,
    pub save_mana: i8,
    pub affect: [u8; MAX_AFFECT],
    pub guild: u16,
    pub guild_level: u16,
    pub resist: [i8; 4],
    pub req_hp: i32,
    pub req_mp: i32,
    pub magic: i32,
    pub rsv: u16,
    pub learned_skill: i8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deku::DekuContainerWrite;

    #[test]
    fn update_score_raw_serialized_size() {
        let raw = UpdateScoreRaw {
            score: ScoreRaw::default(),
            critical: 0,
            save_mana: 0,
            affect: [0; MAX_AFFECT],
            guild: 0,
            guild_level: 0,
            resist: [0; 4],
            req_hp: 0,
            req_mp: 0,
            magic: 0,
            rsv: 0,
            learned_skill: 0,
        };
        assert_eq!(raw.to_bytes().unwrap().len(), 101);
    }
}
