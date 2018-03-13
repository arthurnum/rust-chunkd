extern crate chunk_protocol as protocol;

use member::Member;
use utils::sleep_nop;
use network::Networker;
use protocol::enums::MessageType;

mod member;
mod utils;
mod network;

fn main() {
    protocol::hello();

    let network = Networker::new("127.0.0.1:45000");

    let mut members_list: Vec<Member> = Vec::new();

    loop {
        match network.read() {
            Some(pckg) => {
                let (addr, msg) = pckg;

                match msg {
                    MessageType::AddToListenersRequest => {
                        members_list.push(Member { addr: addr });

                        let data = protocol::pack(&MessageType::ServerOn);
                        network.send_to(&data, &addr);
                    }

                    MessageType::RemoveFromListeners => {
                        members_list.retain(|member| member.addr != addr);
                    }

                    MessageType::MemberIn => {

                    }

                    _ => ()
                }
            }

            None => { sleep_nop(10); }
        }

        // break;
    }
}
