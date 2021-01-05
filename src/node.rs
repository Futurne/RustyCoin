use std::io::{self, Write};
use mio::net::TcpStream;

use crate::messages::states::WhoamiSate;
use crate::messages::header::{HEADER_SIZE, Header};

#[derive(Debug)]
pub struct Node {
    pub connection: TcpStream,
    pub buffer: Vec<u8>,
    pub is_ingoing: bool,

    whoami_state: (WhoamiSate, WhoamiSate),  // (local, remote)
}

impl Node {
    pub fn new(connection: TcpStream, is_ingoing: bool)
        -> Self {
        Node {
            connection,
            buffer: Vec::new(),
            is_ingoing,

            whoami_state: (WhoamiSate::Unkn, WhoamiSate::Unkn),
        }
    }

    pub fn handle_buffer(&mut self) {
        if self.buffer.len() > HEADER_SIZE {
            let (buffer_header, buffer) = self.buffer.split_at(HEADER_SIZE);
            let buffer_header: Vec<u8> = buffer_header.into();
            let header: Header = Header::try_from(buffer_header);
        }
        self.buffer.clear();
    }

    pub fn do_whoami(&mut self) {
        if self.whoami_state.0 == WhoamiSate::Unkn
            && (self.is_ingoing || self.whoami_state.1 == WhoamiSate::Ack) {
            // Send a whoami message to remote node

            self.whoami_state.0 = WhoamiSate::Sent;
        }
    }

    pub fn send_ping(&mut self) -> io::Result<()> {
        let header = Header::new(42, "2plus2is4", 0).unwrap();
        let header: Vec<u8> = Vec::from(header);

        self.connection.write_all(&header)?;
        Ok(())
    }
}
