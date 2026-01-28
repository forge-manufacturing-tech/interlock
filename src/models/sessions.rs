use sea_orm::{entity::prelude::*, ActiveValue};
pub use super::_entities::sessions::{self, ActiveModel, Entity, Model};
use uuid::Uuid;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert && (self.id.is_not_set() || self.id.as_ref().is_nil()) {
            let mut this = self;
            this.id = ActiveValue::Set(Uuid::new_v4());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}
