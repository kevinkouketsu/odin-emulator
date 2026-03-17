use crate::{WritableResource, WritableResourceError, messages::ServerMessage};
use deku::prelude::*;

pub struct UpdateEtc {
    pub experience: i64,
    pub learned_skill: [u32; 2],
    pub score_bonus: i16,
    pub special_bonus: i16,
    pub skill_bonus: i16,
    pub coin: i32,
}

impl WritableResource for UpdateEtc {
    const IDENTIFIER: ServerMessage = ServerMessage::UpdateEtc;
    type Output = UpdateEtcRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(UpdateEtcRaw {
            fake_exp: 0,
            exp: self.experience,
            learned_skill: self.learned_skill,
            score_bonus: self.score_bonus,
            special_bonus: self.special_bonus,
            skill_bonus: self.skill_bonus,
            coin: self.coin,
        })
    }
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct UpdateEtcRaw {
    pub fake_exp: i32,
    pub exp: i64,
    pub learned_skill: [u32; 2],
    pub score_bonus: i16,
    pub special_bonus: i16,
    pub skill_bonus: i16,
    pub coin: i32,
}
