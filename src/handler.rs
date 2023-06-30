use crate::request;
use crate::response;
use crate::room::Room;

use crate::response::{Created, Info, Joined};
use log::info;
use response::{Error, Payload, Signal};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::ws::Message;

type Tx = mpsc::UnboundedSender<Message>;
type Peers = Arc<RwLock<HashMap<Uuid, Tx>>>;
type Rooms = Arc<RwLock<HashMap<Uuid, Room>>>;

#[derive(Clone, Debug)]
pub(crate) struct Handler {
    peers: Peers,
    rooms: Rooms,
}

impl Handler {
    pub(crate) fn new() -> Self {
        Handler {
            peers: Peers::default(),
            rooms: Rooms::default(),
        }
    }

    pub(crate) fn add_peer(&self, tx: Tx) -> Uuid {
        let id = Uuid::new_v4();
        self.peers.write().unwrap().insert(id, tx);
        id
    }

    pub(crate) fn remove_peer(&self, id: &Uuid) {
        self.peers.write().unwrap().remove(id);
        self.rooms.write().unwrap().iter_mut().for_each(|room| {
            room.1.remove_peer(id);
        });
    }

    pub(crate) fn handle(&self, peer: Uuid, msg: Message) {
        if msg.is_close() || msg.is_pong() {
            return;
        }

        if msg.is_ping() {
            self.reply(peer, Message::pong(msg.into_bytes()));
            return;
        }

        match serde_json::from_str::<request::Payload>(msg.to_str().unwrap()) {
            Ok(payload) => self.route(peer, &payload),
            Err(err) => info!("{}", err.to_string()),
        };
    }

    fn route(&self, peer: Uuid, payload: &request::Payload) {
        match payload {
            request::Payload::Create(payload) => self.create(peer, payload),
            request::Payload::Info(payload) => self.info(peer, payload),
            request::Payload::Join(payload) => self.join(peer, payload),
            request::Payload::Signal(payload) => self.signal(peer, payload),
        };
    }

    fn create(&self, peer: Uuid, payload: &request::Create) {
        let mut room = Room::new(payload.data.to_owned());
        room.add_peer(peer);

        self.rooms.write().unwrap().insert(room.id, room.to_owned());
        info!("Created Room: {}", &room.id);

        self.reply(
            peer,
            Message::text(
                serde_json::to_string(&Payload::Created(Created {
                    you: peer,
                    room: room.id.to_owned(),
                }))
                .unwrap(),
            ),
        );
    }

    fn info(&self, peer: Uuid, payload: &request::Info) {
        match self.rooms.read().unwrap().get(&payload.room) {
            None => self.reply(
                peer,
                Message::text(
                    serde_json::to_string(&Payload::Error(Error::RoomDoesNotExists)).unwrap(),
                ),
            ),
            Some(room) => {
                self.reply(
                    peer,
                    Message::text(
                        serde_json::to_string(&Payload::Info(Info {
                            peers: room.peers.len(),
                            data: room.data.clone(),
                        }))
                        .unwrap(),
                    ),
                );
            }
        }
    }

    fn join(&self, peer: Uuid, payload: &request::Join) {
        match self.rooms.write().unwrap().get_mut(&payload.room) {
            None => self.reply(
                peer,
                Message::text(
                    serde_json::to_string(&Payload::Error(Error::RoomDoesNotExists)).unwrap(),
                ),
            ),
            Some(room) => {
                self.reply(
                    peer,
                    Message::text(
                        serde_json::to_string(&Payload::Joined(Joined {
                            you: peer,
                            peers: room.peers.clone(),
                            data: room.data.clone(),
                        }))
                        .unwrap(),
                    ),
                );
                room.add_peer(peer);
            }
        }
    }

    fn signal(&self, peer: Uuid, payload: &request::Signal) {
        self.reply(
            payload.peer,
            Message::text(
                serde_json::to_string(&Payload::Signal(Signal {
                    peer,
                    data: payload.data.to_owned(),
                }))
                .unwrap(),
            ),
        );
    }

    fn reply(&self, to: Uuid, msg: Message) {
        self.peers
            .read()
            .unwrap()
            .get(&to)
            .unwrap()
            .send(msg)
            .unwrap();
    }

    pub(crate) fn clean(&self) {
        self.rooms.write().unwrap().retain(|&_, room| !room.dead());
    }
}
