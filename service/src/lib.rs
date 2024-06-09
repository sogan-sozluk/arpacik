pub mod auth;
pub mod cookie;
pub mod dto;
pub mod entry;
pub mod error;
pub mod feed;
pub mod search;
pub mod title;
pub mod today;
pub mod token;
pub mod trends;
pub mod user;
pub mod validation;

pub use self::error::{Error, Result};
pub use sea_orm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Deleted {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "only")]
    Only,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Author {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "only")]
    Only,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TitleVisible {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "only")]
    Only,
}
