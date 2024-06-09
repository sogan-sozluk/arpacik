use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchItem {
    pub id: i32,
    pub name: String,
}
