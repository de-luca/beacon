use crate::request;
use crate::response;

use futures_channel::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, Arc};
use uuid::Uuid;
use log::info;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<Uuid, Tx>>>;
type RoomMap = Arc<Mutex<HashMap<Uuid, HashSet<Uuid>>>>;


#[derive(Clone, Debug)]
pub(crate) struct Handler {
    peers: PeerMap,
    rooms: RoomMap,
}

impl Handler {
    pub(crate) fn new() -> Self {
        Handler {
            peers: PeerMap::new(Mutex::new(HashMap::new())),
            rooms: RoomMap::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn add_peer(&self, tx: Tx) -> Uuid {
        let id = Uuid::new_v4();
        self.peers.lock().unwrap().insert(id, tx);
        id
    }

    pub(crate) fn remove_peer(&self, id: &Uuid) {
        self.peers.lock().unwrap().remove(&id);
        self.rooms.lock().unwrap().iter_mut().for_each(|room| {
            room.1.remove(&id);
        });
    }

    pub(crate) fn handle(&self, peer_id: Uuid, msg: Message) {
        if msg.is_close() { return }
        if msg.is_pong()  { return }
        if msg.is_ping()  {
            self.peers.lock().unwrap()
                .get(&peer_id).unwrap()
                .unbounded_send(Message::Pong(msg.into_data())).unwrap();
            return
        }

        match serde_json::from_str::<request::Payload>(msg.to_text().unwrap()) {
            Ok(payload) => self.route(peer_id, &payload),
            Err(err) => info!("{}", err.to_string()),
        };
    }

    fn route(&self, peer_id: Uuid, payload: &request::Payload) {
        match payload {
            request::Payload::CREATE(_) => self.create(peer_id),
            request::Payload::JOIN(params) => self.join(peer_id, params),
            request::Payload::SIGNAL(params) => self.signal(peer_id, params),
        };
    }

    fn create(&self, peer_id: Uuid) {
        let room_id = Uuid::new_v4();
        let mut room_peers = HashSet::new();
        room_peers.insert(peer_id);

        self.rooms.lock().unwrap().insert(room_id, room_peers);
        info!("CREATED THE ROOM {}", room_id);

        self.peers.lock().unwrap()
            .get(&peer_id).unwrap()
            .unbounded_send(Message::Text(
                serde_json::to_string(&response::Payload::CREATED(room_id)).unwrap()
            )).unwrap();
    }

    fn join(&self, peer_id: Uuid, params: &request::Join) {
        let locked_peers = self.peers.lock().unwrap();
        let tx = locked_peers.get(&peer_id).unwrap();

        let mut locked_rooms = self.rooms.lock().unwrap();
        let room = locked_rooms.get_mut(&params.room_id);

        match room {
            None => tx.unbounded_send(Message::Text("NOT A ROOM".into())).unwrap(),
            Some(peers) => {
                tx.unbounded_send(Message::Text(
                    serde_json::to_string(&response::Payload::JOINED(peers.to_owned())).unwrap()
                )).unwrap();
                peers.insert(peer_id);
            }
        }
    }

    fn signal(&self, peer_id: Uuid, params: &request::Signal) {
        self.peers.lock().unwrap()
            .get(&params.peer_id).unwrap()
            .unbounded_send(Message::Text(
                serde_json::to_string(&response::Payload::SIGNAL(
                    response::Signal{
                        peer_id,
                        data: params.data.to_owned(),
                    }
                )).unwrap()
            )).unwrap();
    }
}
