extern crate chunk_protocol as protocol;

use std::sync::{Arc, Mutex};
use std::io::{self, Write};

use protocol::enums::MessageType;

mod utils;

mod controllers;

mod rooms;

mod network;
mod router;

fn carriage() { print!("-> "); }
fn help() {
    println!("Commands:\n\tcreate room\n\tclose room [number]\n\tclist\n\tstatus\n\texit");
}

fn main() {
    protocol::hello();
    let network = network::Networker::new("127.0.0.1:45000");
    let mut router = router::Router::new(network);

    let arc_room_counter = Arc::new(Mutex::new(0));
    router.start(&arc_room_counter);

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
                                                    let arc_room = rooms::spawn(room, controllers::Controller{});
                                                    let n = {
                                                        let new_room = arc_room.lock().unwrap();
                                                        *room_conter_lock ^= new_room.flag;
                                                        new_room.number.clone()
                                                    };
                                                    println!("Spawn room {:?}", n);
                                                    router.add_room(n, arc_room);

                                                    let msg = MessageType::RoomStatus { number: n, is_active: true };
                                                    let buf = protocol::pack(&msg);
                                                    for &(addr, _) in router.listeners.lock().unwrap().iter() {
                                                        router.send_to(&buf, &addr);
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
                                                        if router.rooms.get(&room_number).is_some() {
                                                            {
                                                                let arc_room = router.rooms.get(&room_number).unwrap();
                                                                let flag = rooms::close(&arc_room);
                                                                let mut room_conter_lock = arc_room_counter.lock().unwrap();
                                                                *room_conter_lock ^= flag;
                                                            }
                                                            println!("Done");
                                                            router.rooms.remove(&room_number);

                                                            let msg = MessageType::RoomStatus { number: room_number, is_active: false };
                                                            let buf = protocol::pack(&msg);
                                                            for &(addr, _) in router.listeners.lock().unwrap().iter() {
                                                                router.send_to(&buf, &addr);
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
                            for &(addr, rn) in router.listeners.lock().unwrap().iter() { println!("{:?} {:?}", addr, rn); }
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
