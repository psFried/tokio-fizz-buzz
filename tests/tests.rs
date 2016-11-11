extern crate tokio_core;
extern crate futures;

use futures::Future;
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use tokio_core::io::Io;

use std::thread;
use std::time::Duration;
use std::process::{Child, Command};

struct ServerProcess {
    port: u16,
    process: Child,
}

impl ServerProcess {
    pub fn start(port: u16) -> ServerProcess {
        let port_arg = format!("{}", port);
        let child = Command::new("./target/debug/tokio-fizz-buzz").arg(&port_arg).spawn().unwrap();
        thread::sleep(Duration::from_millis(500));
        ServerProcess {
            port: port,
            process: child
        }
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        self.process.kill().unwrap()
    }
}

fn test_fizzbuzz(input: &str, expected_output: &str) {
    let mut core = Core::new().unwrap();
    let address = "127.0.0.1:8080".parse().unwrap();

    let future = TcpStream::connect(&address, &core.handle())
        .and_then(|socket| {
            println!("Got a mutherfucking socket");
            tokio_core::io::write_all(socket, input).and_then(|(socket, _)| {
                println!("wrote: {}", input);
                tokio_core::io::read_exact(socket, Vec::with_capacity(50)).map(|(_, bytes)| {
                    println!("read: {:?}", bytes);
                    let result = String::from_utf8(bytes).unwrap();
                    assert_eq!(expected_output, &result);
                })
            })
        });

    core.run(future).unwrap();
}

#[test]
fn sending_1_returns_1() {
    let process = ServerProcess::start(8080);

    test_fizzbuzz("1", "1");
}
