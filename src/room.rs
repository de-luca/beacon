use std::collections::HashSet;
use std::time::Instant;
use uuid::Uuid;

const GRACE_PERIOD: u64 = 5 * 60;

#[derive(Clone, Debug)]
pub struct Room {
    pub id: Uuid,
    pub peers: HashSet<Uuid>,
    pub data: Option<serde_json::Value>,
    emptied: Option<Instant>,
}

impl Room {
    pub(crate) fn new(data: Option<serde_json::Value>) -> Self {
        Room {
            id: Uuid::new_v4(),
            peers: HashSet::new(),
            data,
            emptied: None,
        }
    }

    fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub(crate) fn dead(&self) -> bool {
        match self.emptied {
            None => false,
            Some(time) => time.elapsed().as_secs() > GRACE_PERIOD,
        }
    }

    pub(crate) fn add_peer(&mut self, id: Uuid) {
        self.peers.insert(id);
    }

    pub(crate) fn remove_peer(&mut self, id: &Uuid) {
        self.peers.remove(id);
        if self.is_empty() {
            self.emptied = Some(Instant::now());
        }
    }
}
