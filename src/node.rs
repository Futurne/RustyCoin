use std::io::{self, Write};
use std::convert::TryFrom;
use mio::net::TcpStream;

use crate::messages::states::*;
use crate::messages::header::Header;
use crate::messages::whoami::Whoami;
use crate::messages::address::Address;
use crate::messages::ByteSize;

/// Represents an exterior node connected to
/// this server.
///
/// All sort of variables are used to precisely
/// define the state of the connection and the informations
/// gathered about this connection.
///
/// The logic behind the protocol is also implemented here.
#[derive(Debug)]
pub struct Node {
    pub connection: TcpStream,
    pub buffer: Vec<u8>,
    pub is_ingoing: bool,  // True if the remote client engaged the connection, false otherwise.
    pub is_valid: bool,  // True if the node is handling all the whoami and ping messages well.

    current_action: CurrentAction,

    whoami_state: (WhoamiSate, WhoamiSate),  // (local, remote)
    pub address: Option<Address>,  // Given by the whoami message
    services: Vec<String>,

    last_ping_sent: u8,
    last_ping_recv: u8,
    last_seen: u32,
    ping_state: PingState,
}

impl Node {
    /// Only needs the connection and the information
    /// of who did the connection.
    pub fn new(connection: TcpStream, is_ingoing: bool)
        -> Self {
        Node {
            connection,
            buffer: Vec::new(),
            is_ingoing,
            is_valid: false,

            current_action: CurrentAction::WaitingHeader,

            whoami_state: (WhoamiSate::Unkn, WhoamiSate::Unkn),
            address: None,
            services: Vec::new(),

            last_ping_sent: PING_CALLBACK,
            last_ping_recv: PING_CALLBACK,
            last_seen: 0,
            ping_state: PingState::Ack,
        }
    }

    /// Recursively process the buffer.
    /// The function is terminated when the node is waiting for a header
    /// and the buffer cannot provides one.
    pub fn handle_buffer(&mut self) {
        self.last_seen = 0;  // Reset the variable (we got a message from the node !)

        match self.current_action {
            CurrentAction::WaitingHeader => if self.do_header() {return;},
            CurrentAction::WaitingWhoami(length) => if self.do_whoami(length as usize) {return;},
        }

        self.handle_buffer();  // Continue working on the buffer (if needed).
    }

    /// Check if a ping is needed to be sent.
    /// Check if we need to send a whoami message.
    pub fn routine(&mut self) {
        if self.last_ping_sent == 0 {
            self.send_ping(PingType::Ping).unwrap();

            self.last_ping_sent = PING_CALLBACK;
            self.ping_state = PingState::Sent;
        }

        if !self.is_ingoing && self.whoami_state.0 == WhoamiSate::Unkn {
            self.send_whoami().expect("Error while sending whoami: ");
        }

        self.is_valid = self.whoami_state.0 == WhoamiSate::Ack
            && self.whoami_state.1 == WhoamiSate::Ack;

        if self.last_seen > LAST_SEEN_THRESHOLD {
            // If above this threshold, we consider the node to be dead.
            println!("The node is not showing any sign of life.");

            // The node is not responding to our ping .. !
            self.is_valid = !(self.ping_state == PingState::Sent);
        }
    }

    /// Parse the header (if possible) and then act properly.
    fn do_header(&mut self) -> bool {
        if let Some((header, buffer)) = Header::read_buffer(&mut self.buffer) {
            self.buffer = buffer;
            if header.magic != MAGIC {
                println!("Wrong magic number");
                return true;
            }

            match header.msg() {
                msg if msg == PING_MSG => {
                    self.send_ping(PingType::Pong).unwrap();
                }
                msg if msg == PONG_MSG => {
                    self.ping_state = PingState::Ack;
                },
                msg if msg == WHOAMI_MSG => {
                    self.current_action = CurrentAction::WaitingWhoami(header.length);
                },
                msg if msg == WHOAMIACK_MSG => {
                    self.whoami_state.0 = WhoamiSate::Ack;
                },
                msg => println!("Header unknown: {}", msg),
            }

            false
        } else {
            // We are waiting for a header, so we need to back off
            // and wait for more incoming messages.
            true  // Escape the `handle_buffer` function.
        }
    }

    /// Parse the Whoami buffer (if big enough).
    /// Send a whoamiack back when everyting is good.
    /// Return true if we need to stop and wait for the buffer to be filled.
    fn do_whoami(&mut self, length: usize) -> bool {
        if self.buffer.len() < length {
            return true;  // Buffer not big enough for the moment
        }

        let whoami = Whoami::try_from(self.buffer.as_slice()).expect("Error while parsing whoami: ");
        self.buffer = self.buffer.split_at(length).1.into();
        self.current_action = CurrentAction::WaitingHeader;

        if whoami.version != VERSION {
            println!("Different versions ! ({} vs {})",
                whoami.version, VERSION);
        }
        self.send_whoamiack().expect("Error while sending whoamiack: ");

        if self.is_ingoing && self.whoami_state.0 == WhoamiSate::Unkn {
            self.send_whoami().expect("Error while sending whoami: ");
        }

        // Process & save infos
        self.address = Some(whoami.from.clone());
        self.services = whoami.services
            .iter()
            .map(|s| s.value())
            .collect();

        false
    }

    /// Send a whoami message to the remote node.
    /// Sets the local `WhoamiState` to `Send`.
    fn send_whoami(&mut self) -> io::Result<()> {
        let services = SERVICES.iter().map(|s| s.to_string()).collect();
        let socket_addr = self.connection.local_addr()?;
        let addr = Address::new(0, socket_addr.ip(), socket_addr.port());
        let whoami = Whoami::new(VERSION, addr, services);

        let header = Header::new(MAGIC, WHOAMI_MSG, whoami.byte_size() as u64).unwrap();
        let header: Vec<u8> = Vec::from(header);
        self.connection.write_all(&header)?;

        let whoami: Vec<u8> = Vec::from(whoami);
        self.connection.write_all(&whoami)?;

        self.whoami_state.0 = WhoamiSate::Sent;
        Ok(())
    }

    /// Send a whoamiack message to the remote node.
    /// Sets the remote `WhoamiState` to `Ack`.
    fn send_whoamiack(&mut self) -> io::Result<()> {
        let header = Header::new(MAGIC, WHOAMIACK_MSG, 0).unwrap();
        let header: Vec<u8> = Vec::from(header);
        self.connection.write_all(&header)?;

        self.whoami_state.1 = WhoamiSate::Ack;
        Ok(())
    }

    fn send_ping(&mut self, ping: PingType) -> io::Result<()> {
        let msg_type = if ping == PingType::Ping {
            PING_MSG
        } else {
            PONG_MSG
        };
        let header = Header::new(MAGIC, msg_type, 0).unwrap();
        let header: Vec<u8> = Vec::from(header);

        self.connection.write_all(&header)?;
        Ok(())
    }

    /// Actualize the timed variables.
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

        self.last_seen += delta as u32;
    }
}
