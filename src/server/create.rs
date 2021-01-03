use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io;

use super::logic;

pub struct Server {
    poll: Poll,
    listener: TcpListener,
    connections: HashMap<Token, TcpStream>,
    server_token: Token,
    unique_token: Token,
}

impl Server {
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
        let connections = HashMap::<Token, TcpStream>::new();

        Ok(Server {
            poll,
            listener,
            connections,
            server_token,
            unique_token,
        })
    }

    pub fn launch(&mut self) -> io::Result<()> {
        // Create storage for events.
        let mut events = Events::with_capacity(128);
        loop {
            self.poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    token if token == self.server_token => self.new_connection()?,
                    token => {
                        let done = if let Some(connection) = self.connections.get_mut(&token) {
                            logic::handle_connection_event(self.poll.registry(), connection, event)?
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

            self.connections.insert(token, connection);
        }

        Ok(())
    }

    fn next_token(&mut self) -> Token {
        let next = self.unique_token.0;
        self.unique_token.0 += 1;
        Token(next)
    }
}
