pub use super::_entities::images::{ActiveModel, Entity, Model};
use crate::models::_entities::images;
use sea_orm::entity::prelude::*;
use sea_orm::SelectColumns;
use std::collections::HashSet;

pub type Images = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {
    pub async fn get_relative_paths<C>(db: &C) -> Result<HashSet<String>, DbErr>
    where
        C: ConnectionTrait,
    {
        let paths: HashSet<String> = Self::find()
            .select_column(images::Column::RelativePath)
            .into_tuple()
            .all(db)
            .await?
            .into_iter()
            .collect();
        Ok(paths)
    }
}
