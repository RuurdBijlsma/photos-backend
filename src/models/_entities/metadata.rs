//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.5

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "metadata")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub exif_tool: Json,
    #[sea_orm(column_type = "JsonBinary")]
    pub file: Json,
    #[sea_orm(column_type = "JsonBinary")]
    pub composite: Json,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub exif: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub xmp: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub mpf: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub jfif: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub icc_profile: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub gif: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub png: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub quicktime: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub matroska: Option<Json>,
    #[sea_orm(unique)]
    pub image_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::images::Entity",
        from = "Column::ImageId",
        to = "super::images::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Images,
}

impl Related<super::images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Images.def()
    }
}
