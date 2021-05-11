use uuid::Uuid;
use serde::{Deserialize};

#[derive(Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "lowercase")]
pub enum Payload {
    CREATE(Create),
    JOIN(Join),
    SIGNAL(Signal),
}

#[derive(Deserialize, Debug)]
pub struct Create {}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Join {
    pub room_id: Uuid,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Signal {
    pub peer_id: Uuid,
    pub payload: serde_json::Value,
}
