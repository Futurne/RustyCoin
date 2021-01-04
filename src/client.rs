mod server;
mod node;

use server::create::Server;

fn main() {
    let mut client = Server::new("127.0.0.1:9000").unwrap();
    let addr = "127.0.0.1:8000".parse().unwrap();

    client.connect(addr);

    client.launch().unwrap();
}
