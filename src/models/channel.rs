use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub owner_id: i64,
    #[serde(default)]
    pub is_private: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMember {
    pub user_id: i64,
    pub username: String,
    pub joined_at: String,
}
