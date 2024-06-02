//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(schema_name = "arpacik", table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub nickname: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub is_moderator: bool,
    pub is_faded: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::downvote::Entity")]
    Downvote,
    #[sea_orm(has_many = "super::entry::Entity")]
    Entry,
    #[sea_orm(has_many = "super::favorite::Entity")]
    Favorite,
    #[sea_orm(has_many = "super::silenced_user::Entity")]
    SilencedUser,
    #[sea_orm(has_many = "super::token::Entity")]
    Token,
    #[sea_orm(has_many = "super::upvote::Entity")]
    Upvote,
}

impl Related<super::downvote::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Downvote.def()
    }
}

impl Related<super::entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Entry.def()
    }
}

impl Related<super::favorite::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Favorite.def()
    }
}

impl Related<super::silenced_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SilencedUser.def()
    }
}

impl Related<super::token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Token.def()
    }
}

impl Related<super::upvote::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Upvote.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
