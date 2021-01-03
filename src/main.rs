mod server;
use server::create::Server;

mod node;

fn main() {
    let mut server = Server::new("127.0.0.1:8000").unwrap();
    server.launch().unwrap();
}
