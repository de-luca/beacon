use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum Payload {
    Create(Create),
    Info(Info),
    Join(Join),
    Signal(Signal),
}

#[derive(Deserialize, Debug)]
pub struct Create {
    pub data: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Info {
    pub room: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct Join {
    pub room: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct Signal {
    pub peer: Uuid,
    pub data: serde_json::Value,
}
