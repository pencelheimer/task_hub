//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.11

use super::sea_orm_active_enums::TaskVisibilityEnum;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub visibility: TaskVisibilityEnum,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::accesses::Entity")]
    Accesses,
    #[sea_orm(has_many = "super::attachments::Entity")]
    Attachments,
}

impl Related<super::accesses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Accesses.def()
    }
}

impl Related<super::attachments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attachments.def()
    }
}
