use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EntityInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsMessage {
    pub event: String,
    pub payload: Payload,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payload {
    pub content: String,
    #[serde(default)]
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Metadata {
    pub model: Option<String>,
    pub tokens_per_second: Option<f64>,
    pub world_id: Option<String>,
    pub character_id: Option<String>,
    pub status: Option<String>,
    pub total_tokens: Option<u32>,
    pub supported_models: Option<Vec<String>>,
    pub available_worlds: Option<Vec<EntityInfo>>,
    pub available_characters: Option<Vec<EntityInfo>>,
    pub available_models: Option<Vec<EntityInfo>>,
}
