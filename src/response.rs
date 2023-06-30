use serde::Serialize;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum Payload {
    Created(Created),
    Info(Info),
    Joined(Joined),
    Signal(Signal),
    Error(Error),
}

#[derive(Serialize, Debug)]
pub struct Created {
    pub you: Uuid,
    pub room: Uuid,
}

#[derive(Serialize, Debug)]
pub struct Info {
    pub peers: usize,
    pub data: Option<serde_json::Value>,
}

#[derive(Serialize, Debug)]
pub struct Joined {
    pub you: Uuid,
    pub data: Option<serde_json::Value>,
    pub peers: HashSet<Uuid>,
}

#[derive(Serialize, Debug)]
pub struct Signal {
    pub peer: Uuid,
    pub data: serde_json::Value,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Error {
    RoomDoesNotExists,
}
