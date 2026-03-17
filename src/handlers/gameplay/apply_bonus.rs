use crate::map::EntityId;
use crate::packets::ToUpdateEtc;
use crate::session::{PacketSender, SessionError};
use crate::world::{Mob, Player, World};
use odin_networking::{WritableResourceError, messages::client::apply_bonus::ApplyBonusRaw};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseStat {
    Strength,
    Intelligence,
    Dexterity,
    Constitution,
}

#[derive(Debug)]
pub struct DistributeScore {
    stat: BaseStat,
}

#[derive(Debug)]
pub struct DistributeMaster {
    index: u8,
}

#[derive(Debug)]
pub enum ApplyBonus {
    Score(DistributeScore),
    Master(DistributeMaster),
}

impl ApplyBonus {
    pub fn handle<P: PacketSender>(
        &self,
        entity_id: EntityId,
        world: &mut World,
        sender: &P,
    ) -> Result<(), ApplyBonusError> {
        {
            let Some(Mob::Player(player)) = world.get_mob_mut(entity_id) else {
                return Err(ApplyBonusError::PlayerNotFound);
            };
            match self {
                ApplyBonus::Score(d) => d.apply(player)?,
                ApplyBonus::Master(d) => d.apply(player)?,
            }
        }
        world.recalculate_score(entity_id);

        let Mob::Player(player) = world.get_mob(entity_id).unwrap();
        sender.send_to(entity_id.id(), player.to_update_etc())?;
        Ok(())
    }
}

impl TryFrom<ApplyBonusRaw> for ApplyBonus {
    type Error = WritableResourceError;

    fn try_from(value: ApplyBonusRaw) -> Result<Self, Self::Error> {
        match value.bonus_type {
            0 => {
                let stat = match value.detail {
                    0 => BaseStat::Strength,
                    1 => BaseStat::Intelligence,
                    2 => BaseStat::Dexterity,
                    3 => BaseStat::Constitution,
                    _ => {
                        return Err(WritableResourceError::Generic(
                            "Invalid apply bonus value".to_string(),
                        ));
                    }
                };
                Ok(ApplyBonus::Score(DistributeScore { stat }))
            }
            1 => {
                if !(0..=3).contains(&value.detail) {
                    return Err(WritableResourceError::Generic(
                        "Invalid apply bonus value".to_string(),
                    ));
                }
                Ok(ApplyBonus::Master(DistributeMaster {
                    index: value.detail as u8,
                }))
            }
            _ => Err(WritableResourceError::Generic(
                "Invalid apply bonus value".to_string(),
            )),
        }
    }
}

impl DistributeScore {
    pub fn apply(&self, player: &mut Player) -> Result<(), ApplyBonusError> {
        if player.computed.score.hp == 0 {
            return Err(ApplyBonusError::Dead);
        }

        if player.score_bonus <= 0 {
            return Err(ApplyBonusError::NoPointsAvailable);
        }

        let add: i16 = if player.score_bonus > 200 { 100 } else { 1 };

        let stat = match self.stat {
            BaseStat::Strength => &mut player.score.strength,
            BaseStat::Intelligence => &mut player.score.intelligence,
            BaseStat::Dexterity => &mut player.score.dexterity,
            BaseStat::Constitution => &mut player.score.constitution,
        };
        *stat += add as u16;
        player.score_bonus -= add;

        Ok(())
    }
}

impl DistributeMaster {
    pub fn apply(&self, player: &mut Player) -> Result<(), ApplyBonusError> {
        if player.computed.score.hp == 0 {
            return Err(ApplyBonusError::Dead);
        }

        if player.special_bonus <= 0 {
            return Err(ApplyBonusError::NoPointsAvailable);
        }

        let cap = ((player.score.level as i32 + 1) * 3) / 2;
        if player.score.specials[self.index as usize] >= cap as u16 {
            return Err(ApplyBonusError::SpecialAtCap);
        }

        player.score.specials[self.index as usize] += 1;
        player.special_bonus -= 1;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ApplyBonusError {
    #[error("Player not found in world")]
    PlayerNotFound,
    #[error("Cannot distribute points while dead")]
    Dead,
    #[error("No bonus points available")]
    NoPointsAvailable,
    #[error("Special stat is at level cap")]
    SpecialAtCap,
    #[error(transparent)]
    PacketSender(#[from] SessionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::tests::MockPacketSender;
    use odin_models::{
        character::{Character, Evolution},
        position::Position,
        status::Score,
        uuid::Uuid,
    };
    use odin_networking::messages::ServerMessage;

    fn make_player(score: Score, evolution: Evolution) -> Player {
        let entity_id = EntityId::Player(1);
        let mut player = Player::from_character(
            entity_id,
            Character {
                identifier: Uuid::new_v4(),
                name: "Test".to_string(),
                score,
                evolution,
                ..Default::default()
            },
        );
        player.computed.score.hp = 100;
        player.calculate_bonus_points();
        player
    }

    fn mortal_player_with_level(level: u16) -> Player {
        make_player(
            Score {
                level,
                ..Default::default()
            },
            Evolution::Mortal,
        )
    }

    #[test]
    fn distribute_score_adds_strength() {
        let mut player = mortal_player_with_level(10);
        let initial_bonus = player.score_bonus;
        let d = DistributeScore {
            stat: BaseStat::Strength,
        };

        d.apply(&mut player).unwrap();

        assert_eq!(player.score.strength, 1);
        assert_eq!(player.score_bonus, initial_bonus - 1);
    }

    #[test]
    fn distribute_score_adds_intelligence() {
        let mut player = mortal_player_with_level(10);
        let d = DistributeScore {
            stat: BaseStat::Intelligence,
        };

        d.apply(&mut player).unwrap();

        assert_eq!(player.score.intelligence, 1);
    }

    #[test]
    fn distribute_score_adds_dexterity() {
        let mut player = mortal_player_with_level(10);
        let d = DistributeScore {
            stat: BaseStat::Dexterity,
        };

        d.apply(&mut player).unwrap();

        assert_eq!(player.score.dexterity, 1);
    }

    #[test]
    fn distribute_score_adds_constitution() {
        let mut player = mortal_player_with_level(10);
        let d = DistributeScore {
            stat: BaseStat::Constitution,
        };

        d.apply(&mut player).unwrap();

        assert_eq!(player.score.constitution, 1);
    }

    #[test]
    fn distribute_score_bulk_when_over_200() {
        // Level 100 mortal TK: score_points=500, base class=25, bonus=525
        let mut player = mortal_player_with_level(100);
        assert!(player.score_bonus > 200);
        let d = DistributeScore {
            stat: BaseStat::Strength,
        };

        d.apply(&mut player).unwrap();

        assert_eq!(player.score.strength, 100);
        assert_eq!(player.score_bonus, 425);
    }

    #[test]
    fn distribute_score_single_when_at_200() {
        // Level 35 mortal TK: score_points=175, base class=25, bonus=200
        let mut player = mortal_player_with_level(35);
        assert_eq!(player.score_bonus, 200);
        let d = DistributeScore {
            stat: BaseStat::Strength,
        };

        d.apply(&mut player).unwrap();

        assert_eq!(player.score.strength, 1);
        assert_eq!(player.score_bonus, 199);
    }

    #[test]
    fn distribute_score_fails_when_dead() {
        let mut player = mortal_player_with_level(10);
        player.computed.score.hp = 0;
        let d = DistributeScore {
            stat: BaseStat::Strength,
        };

        let result = d.apply(&mut player);

        assert_eq!(result, Err(ApplyBonusError::Dead));
    }

    #[test]
    fn distribute_score_fails_when_no_points() {
        // Level 1 mortal TK: score_points=5, base class (8,4,7,6)
        // Set stats to base + 5 distributed = all spent
        let mut player = make_player(
            Score {
                level: 1,
                strength: 13, // 8 base + 5 distributed
                intelligence: 4,
                dexterity: 7,
                constitution: 6,
                ..Default::default()
            },
            Evolution::Mortal,
        );
        assert_eq!(player.score_bonus, 0);
        let d = DistributeScore {
            stat: BaseStat::Strength,
        };

        let result = d.apply(&mut player);

        assert_eq!(result, Err(ApplyBonusError::NoPointsAvailable));
    }

    #[test]
    fn distribute_master_adds_special() {
        // Level 10 mortal = 20 master points
        let mut player = mortal_player_with_level(10);
        let initial_bonus = player.special_bonus;
        let d = DistributeMaster { index: 2 };

        d.apply(&mut player).unwrap();

        assert_eq!(player.score.specials[2], 1);
        assert_eq!(player.special_bonus, initial_bonus - 1);
    }

    #[test]
    fn distribute_master_fails_when_dead() {
        let mut player = mortal_player_with_level(10);
        player.computed.score.hp = 0;
        let d = DistributeMaster { index: 0 };

        let result = d.apply(&mut player);

        assert_eq!(result, Err(ApplyBonusError::Dead));
    }

    #[test]
    fn distribute_master_fails_when_no_points() {
        // Level 1 mortal = 2 master points. Spend all on specials[0].
        let mut player = make_player(
            Score {
                level: 1,
                specials: [2, 0, 0, 0],
                ..Default::default()
            },
            Evolution::Mortal,
        );
        let d = DistributeMaster { index: 0 };

        let result = d.apply(&mut player);

        assert_eq!(result, Err(ApplyBonusError::NoPointsAvailable));
    }

    #[test]
    fn distribute_master_fails_when_at_level_cap() {
        // Level 10 mortal: cap = ((10+1)*3)/2 = 16
        // Give player 16 in specials[1], still has remaining master points
        let mut player = make_player(
            Score {
                level: 10,
                specials: [0, 16, 0, 0],
                ..Default::default()
            },
            Evolution::Mortal,
        );
        let d = DistributeMaster { index: 1 };

        let result = d.apply(&mut player);

        assert_eq!(result, Err(ApplyBonusError::SpecialAtCap));
    }

    #[test]
    fn distribute_master_succeeds_below_cap() {
        // Level 10 mortal: cap = 16, specials[1] = 15 → below cap
        let mut player = make_player(
            Score {
                level: 10,
                specials: [0, 15, 0, 0],
                ..Default::default()
            },
            Evolution::Mortal,
        );
        let d = DistributeMaster { index: 1 };

        d.apply(&mut player).unwrap();

        assert_eq!(player.score.specials[1], 16);
    }

    #[test]
    fn try_from_mode_0_creates_score_variant() {
        let raw = ApplyBonusRaw {
            bonus_type: 0,
            detail: 2,
            target_id: 0,
        };

        let result = ApplyBonus::try_from(raw).unwrap();

        assert!(matches!(
            result,
            ApplyBonus::Score(DistributeScore {
                stat: BaseStat::Dexterity
            })
        ));
    }

    #[test]
    fn try_from_mode_1_creates_master_variant() {
        let raw = ApplyBonusRaw {
            bonus_type: 1,
            detail: 3,
            target_id: 0,
        };

        let result = ApplyBonus::try_from(raw).unwrap();

        assert!(matches!(
            result,
            ApplyBonus::Master(DistributeMaster { index: 3 })
        ));
    }

    #[test]
    fn try_from_invalid_mode_returns_error() {
        let raw = ApplyBonusRaw {
            bonus_type: 2,
            detail: 0,
            target_id: 0,
        };

        assert!(ApplyBonus::try_from(raw).is_err());
    }

    #[test]
    fn try_from_invalid_detail_returns_error() {
        let raw = ApplyBonusRaw {
            bonus_type: 0,
            detail: 5,
            target_id: 0,
        };

        assert!(ApplyBonus::try_from(raw).is_err());
    }

    #[test]
    fn handle_applies_recalculates_and_sends_update_etc() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let entity_id = EntityId::Player(1);
        let character = Character {
            identifier: Uuid::new_v4(),
            name: "Test".to_string(),
            score: Score {
                level: 10,
                hp: 100,
                ..Default::default()
            },
            ..Default::default()
        };
        let player = Player::from_character(entity_id, character);
        world
            .add_player(entity_id, player, Position { x: 2100, y: 2100 })
            .unwrap();
        world.recalculate_score(entity_id);
        {
            let Mob::Player(player) = world.get_mob_mut(entity_id).unwrap();
            player.calculate_bonus_points();
        }

        let bonus = ApplyBonus::Score(DistributeScore {
            stat: BaseStat::Strength,
        });
        bonus.handle(entity_id, &mut world, &sender).unwrap();

        let Mob::Player(player) = world.get_mob(entity_id).unwrap();
        assert_eq!(player.score.strength, 1);
        assert_eq!(player.computed.score.strength, 1);
        // Level 10 TK: score_points=50, base class stats (8+4+7+6)=25
        // Character has 0 initial stats, so spent = 0-25 = -25, bonus = 50+25 = 75
        // After adding 1 str: bonus = 75 - 1 = 74
        assert_eq!(player.score_bonus, 74);

        let messages = sender.messages_for(entity_id);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].identifier, ServerMessage::UpdateEtc);
    }

    #[test]
    fn handle_player_not_found() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let entity_id = EntityId::Player(999);

        let bonus = ApplyBonus::Score(DistributeScore {
            stat: BaseStat::Strength,
        });
        let result = bonus.handle(entity_id, &mut world, &sender);

        assert_eq!(result, Err(ApplyBonusError::PlayerNotFound));
    }
}
