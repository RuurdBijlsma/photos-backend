//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.5

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub filename: String,
    #[sea_orm(unique)]
    pub relative_path: String,
    pub width: i32,
    pub height: i32,
    #[sea_orm(column_type = "Float", nullable)]
    pub duration: Option<f32>,
    pub format: String,
    pub size_bytes: i64,
    pub datetime_local: DateTime,
    pub datetime_utc: Option<DateTime>,
    pub datetime_source: String,
    pub timezone_name: Option<String>,
    pub timezone_offset: Option<String>,
    pub user_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::gps::Entity")]
    Gps,
    #[sea_orm(has_one = "super::metadata::Entity")]
    Metadata,
    #[sea_orm(has_one = "super::tags::Entity")]
    Tags,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
    #[sea_orm(has_many = "super::visual_features::Entity")]
    VisualFeatures,
    #[sea_orm(has_one = "super::weather::Entity")]
    Weather,
}

impl Related<super::gps::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Gps.def()
    }
}

impl Related<super::metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Metadata.def()
    }
}

impl Related<super::tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tags.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::visual_features::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VisualFeatures.def()
    }
}

impl Related<super::weather::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Weather.def()
    }
}
