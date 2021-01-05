// Initialize and handle the main events.
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

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

        println!("Server launched on {}", self.listener.local_addr().unwrap());

        // Main loop
        loop {
            self.poll.poll(&mut events, Some(Duration::from_secs(5)))?;

            for event in events.iter() {
                match event.token() {
                    token if token == self.server_token => self.new_connection()?,
                    token => {
                        let done = if let Some(node) = self.connections.get_mut(&token) {
                            // The event concerns an already connected node.
                            match logic::handle_connection_event(self.poll.registry(), node, event) {
                                Ok(result) => result,
                                Err(err) => {
                                    println!("Closing the connection: {}", err);
                                    true  // Close the connection.
                                }
                            }
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

    /// Connects the server to a specified node.
    /// Registers the node.
    pub fn connect(&mut self, addr: SocketAddr) -> io::Result<()> {
        let connection = TcpStream::connect(addr)?;
        println!("Connected to {}", connection.peer_addr().unwrap());

        // Now register the node
        self.register_node(connection, false)?;
        Ok(())
    }


    /// Accepts and register a new connection.
    fn new_connection(&mut self) -> io::Result<()> {
        loop {
            // Received an event for the TCP server socket, which
            // indicates we can accept an connection.
            let (connection, address) = match self.listener.accept() {
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
            self.register_node(connection, true)?;
        }

        Ok(())
    }

    /// Add a node to the HashMap.
    /// Register the node in the poll for future events.
    fn register_node(&mut self, mut connection: TcpStream, is_ingoing: bool)
            -> io::Result<()> {
        let token = self.next_token();
        self.poll.registry()
            .register(&mut connection, token, Interest::READABLE.add(Interest::WRITABLE))?;

        let node = Node::new(connection, is_ingoing);
        self.connections.insert(token, node);
        Ok(())
    }

    /// Creates a unique token.
    fn next_token(&mut self) -> Token {
        let next = self.unique_token.0;
        self.unique_token.0 += 1;
        Token(next)
    }
}
