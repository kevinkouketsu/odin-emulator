use crate::map::EntityId;
use crate::world::Mob;
use odin_models::position::Position;
use odin_networking::messages::server::create_mob::CreateMob;

pub trait ToCreateMob {
    fn to_create_mob(&self, position: Position) -> CreateMob;
}

impl ToCreateMob for Mob {
    fn to_create_mob(&self, position: Position) -> CreateMob {
        match self {
            Mob::Player(player) => CreateMob {
                position,
                mob_id: player.entity_id().id() as u16,
                name: player.name.clone(),
                score: *player.current_score(),
                equipments: player.equipments.iter().map(|(s, i)| (s, *i)).collect(),
                guild: player.guild,
                guild_level: player.guild_level,
                create_type: 0,
                affect: [0; 32],
            },
        }
    }
}
