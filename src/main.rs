use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::sync::{Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::{self, Write};
use std::time::{Duration};
use std::collections::{HashMap};
use std::collections::hash_map::RandomState;
use std::ops::{Deref, DerefMut};

const ROOM1_FLAG: u8 = 0b00000001;
const ROOM2_FLAG: u8 = 0b00000010;
const ROOM3_FLAG: u8 = 0b00000100;
const ROOM4_FLAG: u8 = 0b00001000;
const CLOSE_ROOM: u8 = 0;
const CONSOLE_DEFAULT: &str = "\x1B[0m";
const CONSOLE_RED: &str = "\x1B[31m";
const CONSOLE_GREEN: &str = "\x1B[32m";

fn sleep_nop(ms: u64) { std::thread::sleep(Duration::from_millis(ms)); }
fn carriage() { print!("-> "); }
fn help() {
    println!("Commands:\n\tcreate room\n\tclose room [number]\n\tstatus\n\texit");
}

fn room_status(n: u8, active: bool) {
    if active { room_active(n); } else { room_closed(n); }
}
fn room_active(n: u8) { println!("Room {}: {}ACTIVE{}", n, CONSOLE_GREEN, CONSOLE_DEFAULT); }
fn room_closed(n: u8) { println!("Room {}: {}CLOSED{}", n, CONSOLE_RED, CONSOLE_DEFAULT); }

fn status(room_counter: &u8) {
    room_status(1, room_counter & ROOM1_FLAG == ROOM1_FLAG);
    room_status(2, room_counter & ROOM2_FLAG == ROOM2_FLAG);
    room_status(3, room_counter & ROOM3_FLAG == ROOM3_FLAG);
    room_status(4, room_counter & ROOM4_FLAG == ROOM4_FLAG);
}

fn create_room<F>(arc: Arc<u8>, closure: F) -> Option<(u8, u8, Sender<u8>, Receiver<u8>)>
    where F: Fn() + Send + 'static + Sync {
    let free_slot = get_free_room_slot(&arc);
    match free_slot {
        Some(slot) => {
            let (flag, number) = slot;
            println!("Spawn room {:?}", number);

            let (p_sender, c_receiver) = channel::<u8>();
            let (c_sender, p_receiver) = channel::<u8>();
            std::thread::spawn(move || {
                loop {
                    closure();
                    std::thread::sleep(Duration::from_millis(2000));
                    let operation = c_receiver.try_recv();
                    if operation.is_ok() {
                        if operation.unwrap() == CLOSE_ROOM { break; }
                    }
                }
                c_sender.send(flag);
            });
            Some((flag, number, p_sender, p_receiver))
        },

        None => {
            println!("No free slots left.");
            None
        }
    }
}

fn close_room(rooms: &mut HashMap<u8, (Sender<u8>, Receiver<u8>)>,
              number: u8,
              mut arc_room_counter: &mut Arc<u8>) {
    if rooms.get(&number).is_some() {
        {
            let &(ref sender, ref receiver) = rooms.get(&number).unwrap();
            sender.send(CLOSE_ROOM);
            println!("Close room {:?}", number);
            loop {
                let operation = receiver.try_recv();
                if operation.is_ok() {
                    *Arc::make_mut(&mut arc_room_counter) ^= operation.unwrap();
                    break;
                }
                println!(" waiting...");
                sleep_nop(500);
            }
        }
        println!("Done");
        rooms.remove(&number);
    } else { status(&arc_room_counter); }
}

fn get_free_room_slot(room_counter: &u8) -> Option<(u8, u8)> {
    if room_counter & ROOM1_FLAG == 0 { return Some((ROOM1_FLAG, 1)) }
    if room_counter & ROOM2_FLAG == 0 { return Some((ROOM2_FLAG, 2)) }
    if room_counter & ROOM3_FLAG == 0 { return Some((ROOM3_FLAG, 3)) }
    if room_counter & ROOM4_FLAG == 0 { return Some((ROOM4_FLAG, 4)) }
    None
}

fn main() {
    let bind_addr = "127.0.0.1:45000".to_string();
    println!("Binding {}", bind_addr);

    let socket = UdpSocket::bind(bind_addr.clone()).expect("couldn't bind to address");
    socket.set_nonblocking(true).expect("couldn't set nonblocking");

    let arc_socket = Arc::new(socket);
    // let arc_socket_shared = arc_socket.clone();

    let mut arc_room_counter = Arc::new(0);
    let mut rooms: HashMap<u8, (Sender<u8>, Receiver<u8>), RandomState> = HashMap::with_capacity(4);

    // let mut buf: [u8; 128] = [0; 128];

    // std::thread::spawn(move || {
        // let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 45001);
        // let ping = String::from("ping");
    //     loop {
    //         arc_socket_shared.send_to(ping.as_bytes(), target).expect("couldn't send a package");
    //         sleep_nop();
    //     }
    // });

    let mut buffer = String::new();
    loop {
        carriage();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).unwrap();
        {
            let mut input = buffer.split_whitespace();
            let command = input.next();
            if command.is_some() {
                match command.unwrap() {

                    "create" => {
                        let arg = input.next();
                        if arg.is_some() {
                            match arg.unwrap() {
                                "room" => {
                                    let arc_socket_shared = arc_socket.clone();
                                    let cr_result = create_room(arc_room_counter.clone(), move || {
                                        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 45001);
                                        let ping = String::from("ping");
                                        arc_socket_shared.send_to(ping.as_bytes(), target).expect("couldn't send a package");
                                    });
                                    if cr_result.is_some() {
                                        let (flag, number, sender, receiver) = cr_result.unwrap();
                                        *Arc::make_mut(&mut arc_room_counter) ^= flag;
                                        rooms.insert(number, (sender, receiver));
                                    }
                                },
                                _ => { help(); }
                            }
                        } else { help(); }
                    },

                    "close" => {
                        let arg = input.next();
                        if arg.is_some() {
                            match arg.unwrap() {
                                "room" => {
                                    let number = input.next();
                                    if number.is_some() {
                                        let val = number.unwrap().parse::<u8>();
                                        if val.is_ok() {
                                            let room_number = val.unwrap();
                                            close_room(&mut rooms, room_number, &mut arc_room_counter);
                                        } else { help(); }
                                    } else { help(); }
                                },
                                _ => { help(); }
                            }
                        } else { help(); }
                    },

                    "status" => { status(&arc_room_counter) }

                    "exit" => { break },

                    _ => { help(); }
                }
            }
        }
        buffer.clear();
        // let recr = arc_socket.recv_from(&mut buf);
        //
        // if recr.is_ok() {
        //     let (_, src_addr) = recr.expect("coundn't read a package");
        // } else { sleep_nop(); }
    }
}
