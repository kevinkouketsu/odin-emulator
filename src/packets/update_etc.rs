use crate::world::Player;
use odin_networking::messages::server::update_etc::UpdateEtc;

pub trait ToUpdateEtc {
    fn to_update_etc(&self) -> UpdateEtc;
}

impl ToUpdateEtc for Player {
    fn to_update_etc(&self) -> UpdateEtc {
        UpdateEtc {
            experience: self.experience,
            learned_skill: [0; 2],
            score_bonus: self.score_bonus,
            special_bonus: self.special_bonus,
            skill_bonus: self.skill_bonus,
            coin: self.coin,
        }
    }
}
