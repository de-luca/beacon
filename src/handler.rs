use crate::request;
use crate::response;
use crate::room::Room;

use futures_channel::mpsc::UnboundedSender;
use log::info;
use response::{Error, Payload, Signal};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<Uuid, Tx>>>;
type RoomMap = Arc<Mutex<HashMap<Uuid, Room>>>;

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
        self.peers.lock().unwrap().remove(id);
        self.rooms.lock().unwrap().iter_mut().for_each(|room| {
            room.1.remove_peer(id);
        });
    }

    pub(crate) fn handle(&self, peer_id: Uuid, msg: Message) {
        if msg.is_close() {
            return;
        }
        if msg.is_pong() {
            return;
        }
        if msg.is_ping() {
            self.peers
                .lock()
                .unwrap()
                .get(&peer_id)
                .unwrap()
                .unbounded_send(Message::Pong(msg.into_data()))
                .unwrap();
            return;
        }

        match serde_json::from_str::<request::Payload>(msg.to_text().unwrap()) {
            Ok(payload) => self.route(peer_id, &payload),
            Err(err) => info!("{}", err.to_string()),
        };
    }

    fn route(&self, peer_id: Uuid, payload: &request::Payload) {
        match payload {
            request::Payload::Create(_) => self.create(peer_id),
            request::Payload::Join(params) => self.join(peer_id, params),
            request::Payload::Signal(params) => self.signal(peer_id, params),
        };
    }

    fn create(&self, peer_id: Uuid) {
        let mut room = Room::new();
        room.add_peer(peer_id);

        self.rooms.lock().unwrap().insert(room.id, room.to_owned());
        info!("CREATED THE ROOM {}", &room.id);

        self.peers
            .lock()
            .unwrap()
            .get(&peer_id)
            .unwrap()
            .unbounded_send(Message::Text(
                serde_json::to_string(&Payload::Created(room.id.to_owned())).unwrap(),
            ))
            .unwrap();
    }

    fn join(&self, peer_id: Uuid, params: &request::Join) {
        let locked_peers = self.peers.lock().unwrap();
        let tx = locked_peers.get(&peer_id).unwrap();

        let mut locked_rooms = self.rooms.lock().unwrap();
        let room = locked_rooms.get_mut(&params.room_id);

        match room {
            None => tx
                .unbounded_send(Message::Text(
                    serde_json::to_string(&Payload::Error(Error::RoomDoesNotExists)).unwrap(),
                ))
                .unwrap(),
            Some(room) => {
                tx.unbounded_send(Message::Text(
                    serde_json::to_string(&Payload::Joined(room.to_owned().peers)).unwrap(),
                ))
                .unwrap();
                room.add_peer(peer_id);
            }
        }
    }

    fn signal(&self, peer_id: Uuid, params: &request::Signal) {
        self.peers
            .lock()
            .unwrap()
            .get(&params.peer_id)
            .unwrap()
            .unbounded_send(Message::Text(
                serde_json::to_string(&Payload::Signal(Signal {
                    peer_id,
                    data: params.data.to_owned(),
                }))
                .unwrap(),
            ))
            .unwrap();
    }

    pub(crate) fn clean(&self) {
        self.rooms.lock().unwrap().retain(|&_, room| !room.dead());
    }
}
