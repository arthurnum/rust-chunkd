extern crate chunk_protocol as protocol;

use std::net::{SocketAddr};
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use std::collections::{HashMap};
use std::collections::hash_map::RandomState;
use std::thread;

use protocol::enums::*;
use protocol::BaseMessage;

mod utils;
use utils::sleep_nop;

mod rooms;
use rooms::{Room};

mod network;

fn carriage() { print!("-> "); }
fn help() {
    println!("Commands:\n\tcreate room\n\tclose room [number]\n\tclist\n\tstatus\n\texit");
}

fn main() {
    protocol::hello();
    let network = network::Networker::new("127.0.0.1:45000");
    let arc_network = Arc::new(Mutex::new(network));

    let listeners: Vec<SocketAddr> = Vec::with_capacity(10);
    let arc_listeners = Arc::new(Mutex::new(listeners));
    let arc_listeners_shared = arc_listeners.clone();

    let mut arc_room_counter = Arc::new(Mutex::new(0));
    let arc_room_counter_shared = arc_room_counter.clone();

    {
        let arc_network_shared = arc_network.clone();

        thread::spawn(move || {
            loop {
                {
                    let mut network_lock = arc_network_shared.lock().unwrap();
                    if network_lock.read() {
                        match network_lock.peek() {
                            Some(msg_type) => {
                                match msg_type {
                                    MessageType::AddToListenersRequest => {
                                        let (addr, _) = network_lock.take();
                                        let mut arc_listeners_lock = arc_listeners_shared.lock().unwrap();
                                        arc_listeners_lock.push(addr);

                                        let room_conter_lock = arc_room_counter_shared.lock().unwrap();
                                        for (number, status) in rooms::rooms_status(&room_conter_lock) {
                                            let msg = protocol::RoomStatusMessage::new(number, status);
                                            let buf = msg.pack();
                                            network_lock.send_to(&buf, &addr);
                                        }
                                    }

                                    MessageType::RemoveFromListeners => {
                                        let (addr, _) = network_lock.take();
                                        let mut arc_listeners_lock = arc_listeners_shared.lock().unwrap();
                                        arc_listeners_lock.retain(|&src| src != addr);
                                    }

                                    _ => ()
                                }
                            },

                            None => ()
                        }
                    }
                }
                sleep_nop(10);
            }
        });
    }

    let mut rooms_hs: HashMap<u8, Arc<Mutex<Room>>, RandomState> = HashMap::with_capacity(4);

    let mut buffer = String::new();
    loop {
        carriage();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).unwrap();
        {
            let input: Vec<&str> = buffer.split_whitespace().collect();

            match input.get(0) {
                Some(command) => {
                    match command as &str {
                        "create" => {
                            match input.get(1) {
                                Some(arg) => {
                                    match arg as &str {
                                        "room" => {
                                            let mut room_conter_lock = arc_room_counter.lock().unwrap();
                                            match rooms::new(&room_conter_lock) {
                                                Some(room) => {
                                                    let arc_room = rooms::spawn(room, move || {
                                                        // empty
                                                    });
                                                    let n = {
                                                        let new_room = arc_room.lock().unwrap();
                                                        *room_conter_lock ^= new_room.flag;
                                                        new_room.number.clone()
                                                    };
                                                    println!("Spawn room {:?}", n);
                                                    rooms_hs.insert(n, arc_room);

                                                    let msg = protocol::RoomStatusMessage::new(n, true);
                                                    let buf = msg.pack();
                                                    let mut network_lock = arc_network.lock().unwrap();
                                                    let arc_listeners_lock = arc_listeners.lock().unwrap();
                                                    for addr in arc_listeners_lock.iter() {
                                                        network_lock.send_to(&buf, addr);
                                                    }
                                                },

                                                None => ()
                                            }
                                        }

                                        _ => { help(); }
                                    }
                                }

                                None => { help(); }
                            }
                        }

                        "close" => {
                            match input.get(1) {
                                Some(arg) => {
                                    match arg as &str {
                                        "room" => {
                                            match input.get(2) {
                                                Some(number) => {
                                                    let val = number.parse::<u8>();
                                                    if val.is_ok() {
                                                        let room_number = val.unwrap();
                                                        if rooms_hs.get(&room_number).is_some() {
                                                            {
                                                                let arc_room = rooms_hs.get(&room_number).unwrap();
                                                                let flag = rooms::close(&arc_room);
                                                                let mut room_conter_lock = arc_room_counter.lock().unwrap();
                                                                *room_conter_lock ^= flag;
                                                            }
                                                            println!("Done");
                                                            rooms_hs.remove(&room_number);

                                                            let msg = protocol::RoomStatusMessage::new(room_number, false);
                                                            let buf = msg.pack();
                                                            let mut network_lock = arc_network.lock().unwrap();
                                                            let arc_listeners_lock = arc_listeners.lock().unwrap();
                                                            for addr in arc_listeners_lock.iter() {
                                                                network_lock.send_to(&buf, addr);
                                                            }
                                                        }
                                                    } else { help(); }
                                                }

                                                None => { help(); }
                                            }
                                        }

                                        _ => { help(); }
                                    }
                                }

                                None => { help(); }
                            }
                        }

                        "clist" => {
                            let arc_listeners_lock = arc_listeners.lock().unwrap();
                            for addr in arc_listeners_lock.iter() { println!("{:?}", addr); }
                        }

                        "status" => {
                            let room_conter_lock = arc_room_counter.lock().unwrap();
                            rooms::status(&room_conter_lock);
                        }

                        "exit" => { break }

                        "test" => {

                        }

                        _ => { help(); }
                    }
                }
                None => ()
            }
        }
        buffer.clear();
    }
}
