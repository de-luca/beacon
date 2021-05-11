use uuid::Uuid;
use serde::{Serialize};
use std::collections::HashSet;

#[derive(Serialize, Debug)]
#[serde(tag = "method", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum Payload {
    CREATE(Uuid),
    JOIN(HashSet<Uuid>),
    SIGNAL(Signal),
}

#[derive(Serialize, Debug)]
pub struct Signal {
    pub peer_id: Uuid,
    pub payload: serde_json::Value,
}
