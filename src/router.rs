use std::net::{SocketAddr};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap};
use std::collections::hash_map::RandomState;
use std::thread;

use network::Networker;
use rooms;
use rooms::Room;
use utils::sleep_nop;
use protocol;
use protocol::enums::MessageType;

type RoomsHashMap = HashMap<u8, Arc<Mutex<Room>>, RandomState>;
type Listener = (SocketAddr, Option<u8>);

pub struct Router {
    network: Arc<Mutex<Networker>>,
    pub rooms: RoomsHashMap,
    pub listeners: Arc<Mutex<Vec<Listener>>>
}

impl Router {
    pub fn new(network: Networker) -> Router {
        let arc_network = Arc::new(Mutex::new(network));
        let rooms = HashMap::with_capacity(4);
        let arc_listeners = Arc::new(Mutex::new(Vec::with_capacity(10)));

        Router {
            network: arc_network,
            rooms: rooms,
            listeners: arc_listeners
        }
    }

    pub fn send_to(&self, buf: &Vec<u8>, addr: &SocketAddr) {
        self.network.lock().unwrap().send_to(&buf, addr);
    }

    pub fn add_room(&mut self, number: u8, arc_room: Arc<Mutex<Room>>) {
        self.rooms.insert(number, arc_room);
    }

    pub fn start(&self, room_counter: &Arc<Mutex<u8>>) {
        let network_shared = self.network.clone();
        let rooms_shared = self.rooms.clone();
        let listeners_shared = self.listeners.clone();
        let room_counter_shared = room_counter.clone();

        thread::spawn(move || {
            loop {
                let package = {
                    let mut network = network_shared.lock().unwrap();
                    if network.read() && network.poll_not_empty() {
                        Some(network.take())
                    } else {
                        None
                    }
                };

                match package {
                    Some(pckg) => {
                        let (addr, msg) = pckg;
                        match msg {
                            MessageType::AddToListenersRequest => {
                                let mut listeners = listeners_shared.lock().unwrap();
                                listeners.push((addr, None));

                                let room_conter_lock = room_counter_shared.lock().unwrap();
                                for (number, status) in rooms::rooms_status(&room_conter_lock) {
                                    let msg = MessageType::RoomStatus { number: number, is_active: status };
                                    let buf = protocol::pack(&msg);
                                    network_shared.lock().unwrap().send_to(&buf, &addr);
                                }
                            }

                            MessageType::RemoveFromListeners => {
                                let mut listeners = listeners_shared.lock().unwrap();
                                listeners.retain(|&(src, _)| src != addr);
                            }

                            MessageType::MemberIn(room_number) => {
                                let mut listeners = listeners_shared.lock().unwrap();
                                match listeners.iter_mut().find(|&&mut(src, _)| src == addr) {
                                    Some(mut listener) => {
                                        listener.1 = Some(room_number)
                                    }

                                    None => ()
                                }
                            }

                            _ => ()
                        }
                    }

                    None => { sleep_nop(10); }
                }
            }
        });
    }
}
