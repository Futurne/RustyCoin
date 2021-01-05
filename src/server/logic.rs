// How the server is supposed to act
use mio::{Registry, Interest};
use mio::event::Event;
use std::io::{self, Read, Write};
use std::str::from_utf8;

use super::super::node::Node;


const DATA: &[u8] = b"Hello world!\n";

pub fn handle_connection_event(
    registry: &Registry,
    node: &mut Node,
    event: &Event,
) -> io::Result<bool> {
    if !event.is_writable() {
        // We close the connection.
        return Err(io::ErrorKind::InvalidInput.into());
    }

    node.send_ping()?;

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
        if let Ok(str_buf) = from_utf8(received_data) {
            println!("Received data: {}", str_buf.trim_end());
        } else {
            println!("Received (none UTF-8) data: {:?}", received_data);
        }

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
