use std::net::SocketAddr;
use mio::net::TcpStream;

pub struct Node {
    pub connection: TcpStream,
    pub address: SocketAddr,
    pub buffer: Vec<u8>,
    pub is_ingoing: bool,
}

impl Node {
}
