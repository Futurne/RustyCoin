use std::net::SocketAddr;
use mio::net::TcpStream;

struct Node {
    connection: TcpStream,
    address: SocketAddr,
    buffer: Vec<u8>,
    is_ingoing: bool,
}
