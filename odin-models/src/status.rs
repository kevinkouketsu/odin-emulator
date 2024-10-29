#[derive(Debug, Default, Clone, Copy)]
pub struct Score {
    pub level: u16,
    pub defense: u32,
    pub damage: u32,
    pub reserved: i8,
    pub attack_run: i8,
    pub max_hp: u32,
    pub max_mp: u32,
    pub hp: u32,
    pub mp: u32,
    pub strength: u16,
    pub intelligence: u16,
    pub dexterity: u16,
    pub constitution: u16,
    pub specials: [u16; 4],
}
