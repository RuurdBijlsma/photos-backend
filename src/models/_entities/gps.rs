//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.5

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "gps")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Float")]
    pub latitude: f32,
    #[sea_orm(column_type = "Float")]
    pub longitude: f32,
    #[sea_orm(column_type = "Float", nullable)]
    pub altitude: Option<f32>,
    pub location_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::locations::Entity",
        from = "Column::LocationId",
        to = "super::locations::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Locations,
}

impl Related<super::locations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Locations.def()
    }
}
