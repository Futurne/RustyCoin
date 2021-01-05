use std::io::{self, Write};
use mio::net::TcpStream;

use crate::messages::states::*;
use crate::messages::header::Header;

#[derive(Debug)]
pub struct Node {
    pub connection: TcpStream,
    pub buffer: Vec<u8>,
    pub is_ingoing: bool,

    whoami_state: (WhoamiSate, WhoamiSate),  // (local, remote)
    current_action: CurrentAction,

    last_ping_sent: u8,
    last_ping_recv: u8,
}

impl Node {
    pub fn new(connection: TcpStream, is_ingoing: bool)
        -> Self {
        Node {
            connection,
            buffer: Vec::new(),
            is_ingoing,

            whoami_state: (WhoamiSate::Unkn, WhoamiSate::Unkn),
            current_action: CurrentAction::Nothing,

            last_ping_sent: PING_CALLBACK,
            last_ping_recv: PING_CALLBACK,
        }
    }

    pub fn handle_buffer(&mut self) {
        if self.current_action == CurrentAction::Nothing {
            if let Some((header, buffer)) = Header::read_buffer(&mut self.buffer) {
                self.buffer = buffer;
                if header.msg() == PING_MSG {  // We just received a ping
                    self.send_ping(PingType::Pong).unwrap();
                }
            }
        }
    }

    pub fn routine(&mut self) {
        if self.last_ping_sent == 0 {
            self.send_ping(PingType::Ping).unwrap();
            self.last_ping_sent = PING_CALLBACK;
        }
    }

    pub fn do_whoami(&mut self) {
        if self.whoami_state.0 == WhoamiSate::Unkn
            && (self.is_ingoing || self.whoami_state.1 == WhoamiSate::Ack) {
            // Send a whoami message to remote node

            self.whoami_state.0 = WhoamiSate::Sent;
        }
    }

    pub fn send_ping(&mut self, ping: PingType) -> io::Result<()> {
        let msg_type = if ping == PingType::Ping {
            PING_MSG
        } else {
            PONG_MSG
        };
        let header = Header::new(42, msg_type, 0).unwrap();
        let header: Vec<u8> = Vec::from(header);

        self.connection.write_all(&header)?;
        Ok(())
    }

    pub fn delta_time(&mut self, delta: u8) {
        self.last_ping_recv = if delta < self.last_ping_recv {
            self.last_ping_recv - delta
        } else {
            0
        };

        self.last_ping_sent = if delta < self.last_ping_sent {
            self.last_ping_sent - delta
        } else {
            0
        };
    }
}
