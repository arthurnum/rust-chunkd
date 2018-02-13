use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use protocol;

type ClientPackage = (SocketAddr, Vec<u8>);

pub struct Networker {
    socket: UdpSocket,
    poll: Vec<ClientPackage>
}

impl Networker {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Networker {
        let socket = UdpSocket::bind(addr).expect("couldn't bind to address");
        socket.set_nonblocking(true).expect("couldn't set nonblocking");

        let poll: Vec<ClientPackage> = Vec::with_capacity(100);

        Networker {
            socket: socket,
            poll: poll
        }
    }

    pub fn read(&mut self) -> bool {
        let mut buf: Vec<u8> = vec![0; 256];
        let result = self.socket.recv_from(&mut buf);

        if result.is_ok() {
            let (_, src_addr) = result.unwrap();
            self.poll.push((src_addr, buf));

            true
        } else { false }
    }

    pub fn peek(&self) -> Option<protocol::enums::MessageType> {
        match self.poll.get(0) {
            Some(pckg) => {
                let result = protocol::get_message_type(&pckg.1);
                if result.is_ok() {
                    Some(result.unwrap())
                } else { None }
            },

            None => { None }
        }
    }

    pub fn take(&mut self) -> ClientPackage { self.poll.remove(0) }

    pub fn send_to(&self, buf: &Vec<u8>, addr: &SocketAddr) {
        self.socket.send_to(&buf, addr).unwrap();
    }
}
