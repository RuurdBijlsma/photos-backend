//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.5

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "face_boxes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub position: Vec<f32>,
    #[sea_orm(column_type = "Float")]
    pub width: f32,
    #[sea_orm(column_type = "Float")]
    pub height: f32,
    pub mouth_left: Vec<f32>,
    pub mouth_right: Vec<f32>,
    pub nose_tip: Vec<f32>,
    pub eye_left: Vec<f32>,
    pub eye_right: Vec<f32>,
    #[sea_orm(column_type = "custom(\"vector\")", select_as = "float4[]")]
    pub embedding: Vec<f32>,
    pub unique_face_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::unique_faces::Entity",
        from = "Column::UniqueFaceId",
        to = "super::unique_faces::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    UniqueFaces,
}

impl Related<super::unique_faces::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UniqueFaces.def()
    }
}
