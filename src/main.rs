extern crate chunk_protocol as protocol;

use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use std::collections::{HashMap};
use std::collections::hash_map::RandomState;
use std::thread;

mod utils;
use utils::sleep_nop;

mod rooms;
use rooms::{Room};

fn carriage() { print!("-> "); }
fn help() {
    println!("Commands:\n\tcreate room\n\tclose room [number]\n\tstatus\n\texit");
}

fn main() {
    protocol::hello();
    let bind_addr = "127.0.0.1:45000".to_string();
    println!("Binding {}", bind_addr);

    let socket = UdpSocket::bind(bind_addr.clone()).expect("couldn't bind to address");
    socket.set_nonblocking(true).expect("couldn't set nonblocking");

    let arc_socket = Arc::new(socket);

    let listeners: Vec<SocketAddr> = Vec::with_capacity(10);
    let arc_listeners = Arc::new(Mutex::new(listeners));

    let mut arc_room_counter = Arc::new(0);
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
                                            match rooms::new(&arc_room_counter) {
                                                Some(room) => {
                                                    let arc_socket_shared = arc_socket.clone();
                                                    let arc_room = rooms::spawn(room, move || {
                                                        // empty
                                                    });
                                                    let n = {
                                                        let new_room = arc_room.lock().unwrap();
                                                        *Arc::make_mut(&mut arc_room_counter) ^= new_room.flag;
                                                        new_room.number.clone()
                                                    };
                                                    println!("Spawn room {:?}", n);
                                                    rooms_hs.insert(n, arc_room);
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
                                                                *Arc::make_mut(&mut arc_room_counter) ^= flag;
                                                            }
                                                            println!("Done");
                                                            rooms_hs.remove(&room_number);
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

                        "status" => { rooms::status(&arc_room_counter); }

                        "exit" => { break }

                        _ => { help(); }
                    }
                }
                None => ()
            }
        }
        buffer.clear();
    }
}
