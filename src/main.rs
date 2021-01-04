mod server;
mod node;

use server::create::Server;


fn main() {
    let mut server = Server::new("127.0.0.1:8000").unwrap();
    server.launch().unwrap();
}
