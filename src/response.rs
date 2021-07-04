use serde::Serialize;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum Payload {
    Created(Uuid),
    Joined(HashSet<Uuid>),
    Signal(Signal),
}

#[derive(Serialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Signal {
    pub peer_id: Uuid,
    pub data: serde_json::Value,
}
