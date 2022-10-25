use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum Payload {
    WhoAmI(WhoAmI),
    Create(Create),
    Join(Join),
    Signal(Signal),
}

#[derive(Deserialize, Debug)]
pub struct WhoAmI {}

#[derive(Deserialize, Debug)]
pub struct Create {}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Join {
    pub room_id: Uuid,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Signal {
    pub peer_id: Uuid,
    pub data: serde_json::Value,
}
