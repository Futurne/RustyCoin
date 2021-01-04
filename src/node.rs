use mio::net::TcpStream;

#[derive(Debug)]
pub struct Node {
    pub connection: TcpStream,
    pub buffer: Vec<u8>,
    pub is_ingoing: bool,
}

impl Node {
    pub fn handle_buffer(&mut self) {
    }
}
