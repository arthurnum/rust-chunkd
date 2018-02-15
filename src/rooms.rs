use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use utils::sleep_nop;
use controllers::Controller;

const ROOM1_FLAG: u8 = 0b00000001;
const ROOM2_FLAG: u8 = 0b00000010;
const ROOM3_FLAG: u8 = 0b00000100;
const ROOM4_FLAG: u8 = 0b00001000;
const CLOSE_ROOM: u8 = 0;
const CONSOLE_DEFAULT: &str = "\x1B[0m";
const CONSOLE_RED: &str = "\x1B[31m";
const CONSOLE_GREEN: &str = "\x1B[32m";

pub struct Room {
    pub number: u8,
    pub flag: u8,
    pub parent_sender: Sender<u8>,
    pub parent_receiver: Receiver<u8>,
    pub child_sender: Sender<u8>,
    pub child_receiver: Receiver<u8>
}

pub fn new(rooms_counter: &u8) -> Option<Room> {
    let free_slot = get_free_room_slot(&rooms_counter);
    match free_slot {
        Some(slot) => {
            let (flag, number) = slot;
            let (ps, cr) = channel::<u8>();
            let (cs, pr) = channel::<u8>();

            Some(Room {
                number: number,
                flag: flag,
                parent_sender: ps,
                parent_receiver: pr,
                child_sender: cs,
                child_receiver: cr
            })
        },

        None => {
            println!("No free slots left.");
            None
        }
    }
}

pub fn spawn(this: Room, controller: Controller) -> Arc<Mutex<Room>> {
    let arc_room = Arc::new(Mutex::new(this));
    let arc_room_shared = arc_room.clone();
    thread::spawn(move || {
        loop {
            // controller call
            sleep_nop(2000);
            let room_lock = arc_room.lock().unwrap();
            let operation = room_lock.child_receiver.try_recv();
            if operation.is_ok() {
                if operation.unwrap() == CLOSE_ROOM { break; }
            }
        }
        let room_lock = arc_room.lock().unwrap();
        room_lock.child_sender.send(room_lock.flag).unwrap();
    });
    arc_room_shared
}

pub fn close(this: &Arc<Mutex<Room>>) -> u8 {
    {
        let lock_room = this.lock().unwrap();
        lock_room.parent_sender.send(CLOSE_ROOM).unwrap();
        println!("Close room {:?}", lock_room.number);
    }
    loop {
        let operation = {
            let lock_room = this.lock().unwrap();
            lock_room.parent_receiver.try_recv()
        };
        if operation.is_ok() { break; }
        println!(" waiting...");
        sleep_nop(500);
    }
    this.lock().unwrap().flag
}

fn get_free_room_slot(rooms_counter: &u8) -> Option<(u8, u8)> {
    if rooms_counter & ROOM1_FLAG == 0 { return Some((ROOM1_FLAG, 1)) }
    if rooms_counter & ROOM2_FLAG == 0 { return Some((ROOM2_FLAG, 2)) }
    if rooms_counter & ROOM3_FLAG == 0 { return Some((ROOM3_FLAG, 3)) }
    if rooms_counter & ROOM4_FLAG == 0 { return Some((ROOM4_FLAG, 4)) }
    None
}

pub fn rooms_status(room_counter: &u8) -> Vec<(u8, bool)> {
    vec![
        (1, room_counter & ROOM1_FLAG == ROOM1_FLAG),
        (2, room_counter & ROOM2_FLAG == ROOM2_FLAG),
        (3, room_counter & ROOM3_FLAG == ROOM3_FLAG),
        (4, room_counter & ROOM4_FLAG == ROOM4_FLAG)
    ]
}

pub fn status(room_counter: &u8) {
    room_status(1, room_counter & ROOM1_FLAG == ROOM1_FLAG);
    room_status(2, room_counter & ROOM2_FLAG == ROOM2_FLAG);
    room_status(3, room_counter & ROOM3_FLAG == ROOM3_FLAG);
    room_status(4, room_counter & ROOM4_FLAG == ROOM4_FLAG);
}

fn room_status(n: u8, active: bool) {
    if active { room_active(n); } else { room_closed(n); }
}
fn room_active(n: u8) { println!("Room {}: {}ACTIVE{}", n, CONSOLE_GREEN, CONSOLE_DEFAULT); }
fn room_closed(n: u8) { println!("Room {}: {}CLOSED{}", n, CONSOLE_RED, CONSOLE_DEFAULT); }
