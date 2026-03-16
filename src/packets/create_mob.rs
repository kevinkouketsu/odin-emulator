use crate::map::EntityId;
use crate::world::Mob;
use odin_models::position::Position;
use odin_networking::messages::server::create_mob::CreateMob;

pub trait ToCreateMob {
    fn to_create_mob(&self, entity_id: EntityId, position: Position) -> CreateMob;
}

impl ToCreateMob for Mob {
    fn to_create_mob(&self, entity_id: EntityId, position: Position) -> CreateMob {
        let mob_id = entity_id.id() as u16;
        match self {
            Mob::Player(player) => {
                let base_character = player.base_character();
                CreateMob {
                    position,
                    mob_id,
                    name: base_character.name.clone(),
                    score: *player.current_score(),
                    equipments: base_character.equipments.clone(),
                    guild: base_character.guild,
                    guild_level: base_character.guild_level,
                    create_type: 0,
                    affect: [0; 32],
                }
            }
        }
    }
}
