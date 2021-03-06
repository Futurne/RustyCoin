mod server;
mod node;
mod messages;

use server::Server;

fn main() {
    let mut client = Server::new("127.0.0.1:9000").unwrap();
    let addr = "127.0.0.1:8000".parse().unwrap();

    client.connect(addr).unwrap();

    client.launch().unwrap();
}
