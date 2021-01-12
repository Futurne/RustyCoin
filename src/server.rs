// Contain all server's oriented functions.
use std::collections::HashMap;
use std::io::{self, Read};
use std::net::SocketAddr;
use std::time::Duration;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use crate::node::Node;

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
    /// Permanently listens for new connections, and calls `handle_connection_event`
    /// if an event concerns an already connected node.
    pub fn launch(&mut self) -> io::Result<()> {
        // Create storage for events.
        let mut events = Events::with_capacity(128);
        const WAITING_TIME: u8 = 5;

        println!("Server launched on {}", self.listener.local_addr().unwrap());

        // Main loop
        loop {
            self.poll.poll(&mut events, Some(Duration::from_secs(WAITING_TIME.into())))?;

            for event in events.iter() {
                match event.token() {
                    token if token == self.server_token => self.new_connection()?,
                    token => {
                        let done = if let Some(node) = self.connections.get_mut(&token) {
                            // The event concerns an already connected node.
                            match handle_incoming_messages(node) {
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
            // End of events handling.

            // We now scan all nodes and do the routines.
            for node in self.connections.values_mut() {
                node.delta_time(WAITING_TIME);
                node.routine();
            }
        }
    }

    /// Connects the server to a specified node.
    /// Registers the node.
    #[allow(dead_code)]
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

/// Read and store the incoming messages into the
/// node's buffer.
///
/// Calls the handling buffer's node function.
pub fn handle_incoming_messages(
    node: &mut Node,
) -> io::Result<bool> {
    let mut connection_closed = false;
    let mut received_data = vec![0; 4096];
    let mut bytes_read = 0;

    // We can (maybe) read from the connection.
    loop {
        match node.connection.read(&mut received_data[bytes_read..]) {
            Ok(0) => {
                // Reading 0 bytes means the other side has closed the
                // connection or is done writing, then so are we.
                connection_closed = true;
                break;
            }
            Ok(n) => {
                bytes_read += n;
                if bytes_read == received_data.len() {
                    received_data.resize(received_data.len() + 1024, 0);
                }
            }
            // Would block "errors" are the OS's way of saying that the
            // connection is not actually ready to perform this I/O operation.
            Err(ref err) if would_block(err) => break,
            Err(ref err) if interrupted(err) => continue,
            // Other errors we'll consider fatal.
            Err(err) => return Err(err),
        }
    }

    if bytes_read != 0 {
        let received_data = &received_data[..bytes_read];
        node.buffer.extend(received_data);
        node.handle_buffer();
    }

    if connection_closed {
        println!("Connection with node {} closed.",
            node.connection.peer_addr().unwrap());
        return Ok(true);
    }

    Ok(false)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
