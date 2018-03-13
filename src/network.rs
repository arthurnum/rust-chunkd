use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use protocol;
use protocol::enums::MessageType;

type ClientPackage = (SocketAddr, MessageType);

pub struct Networker {
    socket: UdpSocket
}

impl Networker {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Networker {
        let socket = UdpSocket::bind(addr).expect("couldn't bind to address");
        socket.set_nonblocking(true).expect("couldn't set nonblocking");

        Networker {
            socket: socket
        }
    }

    pub fn read(&self) -> Option<ClientPackage> {
        let mut buf: Vec<u8> = vec![0; 256];
        let result = self.socket.recv_from(&mut buf);

        if result.is_ok() {
            let (_, src_addr) = result.unwrap();
            Some((src_addr, protocol::unpack(&buf)))
        } else { None }
    }

    pub fn send_to(&self, buf: &Vec<u8>, addr: &SocketAddr) {
        self.socket.send_to(&buf, addr).unwrap();
    }
}
