use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Account {
    pub username: String,
    pub password: String,
    pub ban: Option<Ban>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ban {
    pub expiration: NaiveDateTime,
    pub r#type: BanType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BanType {
    Analysis,
    Blocked,
}
