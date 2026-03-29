pub mod loading;
pub mod mob_id_allocator;
pub mod movement;
pub mod pathfinding;
pub mod spawn_group;
pub mod spawn_manager;
pub mod tick;

use crate::map::EntityId;
use crate::score::ComputedScore;
use movement::MovementState;
use odin_models::EquipmentSlots;
use odin_models::character::{Class, GuildLevel};
use odin_models::npc_mob::NpcMob;
use odin_models::status::Score;

pub struct Npc {
    pub entity_id: EntityId,
    pub template: NpcMob,
    pub movement: MovementState,
    pub computed: ComputedScore,
    pub group_id: Option<usize>,
    pub is_leader: bool,
    pub leader: Option<EntityId>,
}

impl Npc {
    pub fn new(entity_id: EntityId, template: NpcMob, movement: MovementState) -> Self {
        let computed = ComputedScore {
            score: template.score,
            ..Default::default()
        };
        Self {
            entity_id,
            template,
            movement,
            computed,
            group_id: None,
            is_leader: false,
            leader: None,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn name(&self) -> &str {
        &self.template.name
    }

    pub fn current_score(&self) -> &Score {
        &self.computed.score
    }

    pub fn equipments(&self) -> &EquipmentSlots {
        &self.template.equipments
    }

    pub fn class(&self) -> Class {
        self.template.class
    }

    pub fn guild(&self) -> Option<i16> {
        self.template.guild
    }

    pub fn guild_level(&self) -> Option<GuildLevel> {
        self.template.guild_level
    }
}
