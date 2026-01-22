//! `SeaORM` Entity.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    #[serde(with = "self::i64_as_string")]
    pub id: i64,
    pub title: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub content: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

mod i64_as_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(val: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        i64::from_str(&s).map_err(serde::de::Error::custom)
    }
}

