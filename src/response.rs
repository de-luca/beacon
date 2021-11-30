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
    Error(Error),
}

#[derive(Serialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Signal {
    pub peer_id: Uuid,
    pub data: serde_json::Value,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Error {
    RoomDoesNotExists,
}
