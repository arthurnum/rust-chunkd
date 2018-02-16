use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use protocol;
use protocol::enums::MessageType;

type ClientPackage = (SocketAddr, MessageType);

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
            self.poll.push((src_addr, protocol::unpack(&buf)));

            true
        } else { false }
    }

    pub fn take(&mut self) -> ClientPackage { self.poll.remove(0) }
    pub fn poll_not_empty(&self) -> bool { !self.poll.is_empty() }

    pub fn send_to(&self, buf: &Vec<u8>, addr: &SocketAddr) {
        self.socket.send_to(&buf, addr).unwrap();
    }
}
