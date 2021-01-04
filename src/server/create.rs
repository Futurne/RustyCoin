// Initialize and handle the main events.
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io;

use super::logic;
use super::super::node::Node;

/// Representation of the server.
///
/// A connected node is registered into the HashMap<Token, Node>.
pub struct Server {
    poll: Poll,
    listener: TcpListener,
    connections: HashMap<Token, Node>,
    server_token: Token,
    unique_token: Token,
}

impl Server {
    /// Creates a new server.
    ///
    /// The server is hosted on the given address.
    pub fn new(addr: &str) -> io::Result<Self> {
        // Unique token for each incoming connection.
        let server_token = Token(0);
        let unique_token = Token(server_token.0 + 1);

        // Create a poll instance.
        let poll = Poll::new()?;

        let mut listener = TcpListener::bind(addr.parse().unwrap())?;

        // Register the server with poll we can receive events for it.
        poll.registry()
            .register(&mut listener, server_token, Interest::READABLE)?;

        // Map of `Token` -> `TcpStream`.
        let connections = HashMap::<Token, Node>::new();

        Ok(Server {
            poll,
            listener,
            connections,
            server_token,
            unique_token,
        })
    }

    /// Launch the main loop of the server.
    ///
    /// Permanently listens for new connections, and calls `logic::handle_connection_event`
    /// if an event concerns an already connected node.
    pub fn launch(&mut self) -> io::Result<()> {
        // Create storage for events.
        let mut events = Events::with_capacity(128);

        // Main loop
        loop {
            self.poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    token if token == self.server_token => self.new_connection()?,
                    token => {
                        let done = if let Some(node) = self.connections.get_mut(&token) {
                            // The event concerns an already connected node.
                            logic::handle_connection_event(self.poll.registry(), node, event)?
                        } else {
                            // Sporadic events happen, we can safely ignore them.
                            false
                        };

                        if done {
                            self.connections.remove(&token); // Disconnect
                        }
                    }
                }
            }
        }
    }

    fn new_connection(&mut self) -> io::Result<()> {
        loop {
            // Received an event for the TCP server socket, which
            // indicates we can accept an connection.
            let (mut connection, address) = match self.listener.accept() {
                Ok((connection, address)) => (connection, address),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // If we get a `WouldBlock` error we know our
                    // listener has no more incoming connections queued,
                    // so we can return to polling and wait for some
                    // more.
                    break;
                }
                Err(e) => {
                    // If it was any other kind of error, something went
                    // wrong and we terminate with an error.
                    return Err(e);
                }
            };

            println!("Accepted connection from: {}", address);

            let token = self.next_token();
            self.poll.registry().register(
                &mut connection,
                token,
                Interest::READABLE.add(Interest::WRITABLE),
            )?;

            let node = Node{connection, address, buffer: Vec::new(), is_ingoing: true};
            self.connections.insert(token, node);
        }

        Ok(())
    }

    fn next_token(&mut self) -> Token {
        let next = self.unique_token.0;
        self.unique_token.0 += 1;
        Token(next)
    }
}