extern crate chunk_protocol as protocol;
extern crate cgmath;
extern crate time;

use std::collections::HashMap;
use std::net::SocketAddr;

use member::Member;
use utils::sleep_nop;
use network::Networker;
use protocol::enums::MessageType;

mod member;
mod utils;
mod network;
mod timers;

fn main() {
    protocol::hello();

    let network = Networker::new("127.0.0.1:45000");

    let mut members: HashMap<SocketAddr, Member> = HashMap::new();

    loop {
        match network.read() {
            Some(pckg) => {
                let (addr, msg) = pckg;

                match msg {
                    MessageType::AddToListenersRequest => {
                        members.insert(addr, Member::default(addr));

                        let data = protocol::pack(&MessageType::ServerOn);
                        network.send_to(&data, &addr);
                    }

                    MessageType::RemoveFromListeners => {
                        members.remove(&addr);
                    }

                    MessageType::MemberIn => match members.get_mut(&addr) {
                        Some(member) => {
                            member.session_on = true;
                        }

                        None => (),
                    }

                    MessageType::MemberMove(x, y) => match members.get_mut(&addr) {
                        Some(member) => {
                            member.moving = true;
                            member.move_direction.x = x;
                            member.move_direction.y = y;
                            if member.debug_move_start == 0 {
                                member.debug_move_start = member.timer.elapsed();
                            }
                        }

                        None => (),
                    }

                    MessageType::MemberStopMove => match members.get_mut(&addr) {
                        Some(member) => {
                            member.moving = false;
                            member.debug_move_stop = member.timer.elapsed();
                            println!("Moving elapsed time {:?}", member.debug_move_stop - member.debug_move_start);
                            member.debug_move_start = 0
                        }

                        None => (),
                    }

                    _ => (),
                }
            }

            None => {
                sleep_nop(10);
            }
        }

        for member in members.values_mut() {
            member.update();
        }
    }
}
