extern crate futures;
extern crate tokio_core;

use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_core::net::TcpStream;

fn main() {
    let port = std::env::args().nth(1).expect("Expected a port argument");

    let mut core = Core::new().unwrap();
    let mut address_str = "127.0.0.1:".to_owned();
    address_str.push_str(&port);

    let address = address_str.parse().unwrap();
    let listener = TcpListener::bind(&address, &core.handle()).unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening for connections on {}", addr);

    let clients = listener.incoming();
    let welcomes = clients.and_then(|(socket, peer_addr)| {
        println!("Got connection from: {:?}", peer_addr);
        tokio_core::io::write_all(socket, b"Hello!\n")
    });
    let server = welcomes.for_each(|(_socket, _welcome)| {
        println!("finished writing maybe?");
        Ok(())
    });

    core.run(server).unwrap();
}

