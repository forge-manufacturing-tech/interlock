//! `SeaORM` Entity.

use sea_orm::{entity::prelude::*, RelationTrait};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[sea_orm(table_name = "blobs")]
pub struct Model {
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTimeWithTimeZone,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub session_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub size: i64,
    pub storage_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sessions::Entity",
        from = "Column::SessionId",
        to = "super::sessions::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Session,
}

impl Related<super::sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}
