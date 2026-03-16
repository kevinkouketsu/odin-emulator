use crate::effect::Effect;
use thiserror::Error;

pub const MAX_ITEM_DATA_EFFECTS: usize = 12;

#[derive(Default)]
pub struct ItemDatabase {
    items: Vec<Option<ItemData>>,
}

impl ItemDatabase {
    pub fn get(&self, id: u16) -> Option<&ItemData> {
        self.items.get(id as usize)?.as_ref()
    }

    pub fn len(&self) -> usize {
        self.items.iter().filter(|i| i.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|i| i.is_none())
    }

    pub fn from_items(items: impl IntoIterator<Item = ItemData>) -> Self {
        let mut db = Self { items: Vec::new() };
        for item in items {
            db.insert(item);
        }
        db
    }

    pub fn from_csv(contents: &str) -> Result<Self, ItemDataParseError> {
        let mut db = Self { items: Vec::new() };

        for (line_num, line) in contents.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() < 9 {
                continue;
            }

            let item_id: u16 = parts[0]
                .parse()
                .map_err(|_| ItemDataParseError::InvalidLine {
                    line: line_num + 1,
                    reason: "invalid item ID".to_string(),
                })?;

            if item_id == 0 {
                continue;
            }

            let name = parts[1].to_string();
            let mesh = parse_mesh(parts[2]).map_err(|_| ItemDataParseError::InvalidLine {
                line: line_num + 1,
                reason: "invalid mesh".to_string(),
            })?;

            let score_parts = parse_score_requirements(parts[3]).map_err(|_| {
                ItemDataParseError::InvalidLine {
                    line: line_num + 1,
                    reason: "invalid score requirements".to_string(),
                }
            })?;

            let unique: i16 = parts[4]
                .parse()
                .map_err(|_| ItemDataParseError::InvalidLine {
                    line: line_num + 1,
                    reason: "invalid unique".to_string(),
                })?;

            let price: i32 = parts[5]
                .parse()
                .map_err(|_| ItemDataParseError::InvalidLine {
                    line: line_num + 1,
                    reason: "invalid price".to_string(),
                })?;

            let pos: i32 = parts[6]
                .parse()
                .map_err(|_| ItemDataParseError::InvalidLine {
                    line: line_num + 1,
                    reason: "invalid pos".to_string(),
                })?;

            let extreme: i16 = parts[7]
                .parse()
                .map_err(|_| ItemDataParseError::InvalidLine {
                    line: line_num + 1,
                    reason: "invalid extreme".to_string(),
                })?;

            let grade: i16 = parts[8]
                .parse()
                .map_err(|_| ItemDataParseError::InvalidLine {
                    line: line_num + 1,
                    reason: "invalid grade".to_string(),
                })?;

            let mut effects = [ItemDataEffect::default(); MAX_ITEM_DATA_EFFECTS];
            let effect_start = 9;
            for (i, effect) in effects.iter_mut().enumerate() {
                let name_idx = effect_start + i * 2;
                let val_idx = name_idx + 1;

                if val_idx >= parts.len() {
                    break;
                }

                let effect_name = parts[name_idx].trim();
                let effect_value: i16 =
                    parts[val_idx]
                        .parse()
                        .map_err(|_| ItemDataParseError::InvalidLine {
                            line: line_num + 1,
                            reason: format!("invalid effect value at position {}", i),
                        })?;

                let index = match Effect::from_name(effect_name) {
                    Some(e) => e as u8,
                    None => {
                        let fallback = effect_name.parse::<u8>().unwrap_or(0);
                        if fallback == 0 && !effect_name.is_empty() && effect_name != "0" {
                            log::warn!(
                                "Unknown effect '{}' on item {} (line {}), defaulting to 0",
                                effect_name,
                                item_id,
                                line_num + 1
                            );
                        }
                        fallback
                    }
                };

                *effect = ItemDataEffect {
                    index,
                    value: effect_value,
                };
            }

            db.insert(ItemData {
                id: item_id,
                name,
                mesh,
                level: score_parts.0,
                str_req: score_parts.1,
                int_req: score_parts.2,
                dex_req: score_parts.3,
                con_req: score_parts.4,
                effects,
                price,
                unique,
                pos,
                extreme,
                grade,
            });
        }

        Ok(db)
    }

    fn insert(&mut self, item: ItemData) {
        let id = item.id as usize;
        if id >= self.items.len() {
            self.items.resize(id + 1, None);
        }
        self.items[id] = Some(item);
    }
}

fn parse_mesh(s: &str) -> Result<(i16, i32), ()> {
    let mut parts = s.split('.');
    let mesh1: i16 = parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let mesh2: i32 = parts.next().unwrap_or("0").parse().map_err(|_| ())?;
    Ok((mesh1, mesh2))
}

fn parse_score_requirements(s: &str) -> Result<(i16, i16, i16, i16, i16), ()> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 5 {
        return Err(());
    }
    Ok((
        parts[0].parse().map_err(|_| ())?,
        parts[1].parse().map_err(|_| ())?,
        parts[2].parse().map_err(|_| ())?,
        parts[3].parse().map_err(|_| ())?,
        parts[4].parse().map_err(|_| ())?,
    ))
}

#[derive(Debug, Clone)]
pub struct ItemData {
    pub id: u16,
    pub name: String,
    pub mesh: (i16, i32),
    pub level: i16,
    pub str_req: i16,
    pub int_req: i16,
    pub dex_req: i16,
    pub con_req: i16,
    pub effects: [ItemDataEffect; MAX_ITEM_DATA_EFFECTS],
    pub price: i32,
    pub unique: i16,
    pub pos: i32,
    pub extreme: i16,
    pub grade: i16,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ItemDataEffect {
    pub index: u8,
    pub value: i16,
}

#[derive(Debug, Error)]
pub enum ItemDataParseError {
    #[error("Invalid line {line}: {reason}")]
    InvalidLine { line: usize, reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sword_item() -> ItemData {
        let mut effects = [ItemDataEffect::default(); MAX_ITEM_DATA_EFFECTS];
        effects[0] = ItemDataEffect {
            index: Effect::Damage as u8,
            value: 50,
        };
        effects[1] = ItemDataEffect {
            index: Effect::Str as u8,
            value: 10,
        };

        ItemData {
            id: 100,
            name: "Sword".to_string(),
            mesh: (1, 0),
            level: 10,
            str_req: 20,
            int_req: 0,
            dex_req: 0,
            con_req: 0,
            effects,
            price: 1000,
            unique: 41,
            pos: 64,
            extreme: 0,
            grade: 0,
        }
    }

    #[test]
    fn get_returns_item_by_id() {
        let db = ItemDatabase::from_items([sword_item()]);
        let item = db.get(100).unwrap();
        assert_eq!(item.name, "Sword");
        assert_eq!(item.id, 100);
    }

    #[test]
    fn get_returns_none_for_missing_id() {
        let db = ItemDatabase::from_items([sword_item()]);
        assert!(db.get(0).is_none());
        assert!(db.get(999).is_none());
    }

    #[test]
    fn default_is_empty() {
        let db = ItemDatabase::default();
        assert!(db.get(1).is_none());
        assert!(db.is_empty());
    }

    #[test]
    fn len_counts_items() {
        let db = ItemDatabase::from_items([sword_item()]);
        assert_eq!(db.len(), 1);
        assert!(!db.is_empty());
    }

    #[test]
    fn from_csv_parses_basic_line() {
        let csv = "100, Sword, 1.0, 10.20.0.0.0, 41, 1000, 64, 0, 0, EF_DAMAGE, 50, EF_STR, 10";
        let db = ItemDatabase::from_csv(csv).unwrap();
        let item = db.get(100).unwrap();
        assert_eq!(item.name, "Sword");
        assert_eq!(item.unique, 41);
        assert_eq!(item.pos, 64);
        assert_eq!(item.level, 10);
        assert_eq!(item.str_req, 20);
        assert_eq!(item.effects[0].index, Effect::Damage as u8);
        assert_eq!(item.effects[0].value, 50);
        assert_eq!(item.effects[1].index, Effect::Str as u8);
        assert_eq!(item.effects[1].value, 10);
    }

    #[test]
    fn from_csv_skips_comments_and_empty_lines() {
        let csv = "# comment\n\n100, Sword, 1.0, 10.0.0.0.0, 0, 0, 64, 0, 0\n";
        let db = ItemDatabase::from_csv(csv).unwrap();
        assert!(db.get(100).is_some());
    }

    #[test]
    fn from_csv_handles_empty_fields_between_commas() {
        let csv = "100,,1.0,10.0.0.0.0,0,0,64,0,0";
        let result = ItemDatabase::from_csv(csv);
        let db = result.unwrap();
        let item = db.get(100).unwrap();
        assert_eq!(item.name, "");
    }

    #[test]
    fn from_csv_parses_multiple_items() {
        let csv = "100, Sword, 1.0, 10.0.0.0.0, 41, 1000, 64, 0, 0\n\
                   200, Shield, 2.0, 5.0.0.0.0, 42, 500, 128, 0, 0";
        let db = ItemDatabase::from_csv(csv).unwrap();
        assert_eq!(db.len(), 2);
        assert_eq!(db.get(100).unwrap().name, "Sword");
        assert_eq!(db.get(200).unwrap().name, "Shield");
    }
}
