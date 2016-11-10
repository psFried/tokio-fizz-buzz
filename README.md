Async IO in Rust
================

## Foundation

- **[Mio](https://github.com/carllerche/mio)**
    - Provides abstraction over OS-specific async IO features
    - Event loop is defined here
- **[Futures](https://github.com/alexcrichton/futures-rs)**
    - Abstraction over some asynchronous unit of work
    - Allows asynchronous units of computation to be composed!
    - Futures crate primarily contains only abstractions
    - Different from other Futures libraries in that this crate uses a pull model
- **[Tokio-Core](https://github.com/tokio-rs/tokio-core)**
    - Depends on both Futures and Mio crates
    - Provides concrete implementations of Futures for basic things
- **[Tokio-Service](https://github.com/tokio-rs/tokio-service)**
    - Depends on Tokio-Core
    - Defines a Service trait that allows dealing with things in terms of Request/Response
    - Tokio-Service only contains abstractions
- **[Tokio-Hyper](https://github.com/tokio-rs/tokio-hyper)**
    - Provides a Tokio Service for HTTP Request-Response
    - Still in kind of a proof-of-concept state
    
## Mio

Mio is a fairly low-level library that abstracts over platform-specific async io primitives. This provides the basis for all of the async io in Tokio. Mio by itself is definitely lower level than you'd typically want for doing things like making http requests and stuff like that.
    
## Futures

The futures crate provides a few primary abstractions:

- **Future**
    - Represents some asynchronous unit of work that may complete at some point in the... future
    - It's a monad, you can flat map that shit! This allows them to be very easily composable
    - All basic io methods in `tokio_core::io` return `Future`s. For instance reading or writing some bytes
    - Primary method is `fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error>`
        - `Async<T>` has two variants: `Ready(T)` and `NotReady`
- **Stream**
    - Represents an asynchronous sequential iterator of Futures
    - `into_future` returns a Future that resolves to the next element in the Stream, plus the Stream itself
    - `TCPListener::incoming()` returns a `Stream` of `TCPStream`s
- **Task**
    - WTF? See below
    - Provides `park` and `unpark` methods. Starts to sort of be like green threads
    
### WTF is a Task?

Most folks follow along just fine until the `Task` comes into play. This is where Rust's Futures are different from what you're probably used to. Most implementations of Futures use a 'push' model. In a push model, there is typically a callback that is invoked once the Future is resolved to a value. The 'pull' model is just the opposite. Instead of invoking a callback once the Future is completed, the Future is polled to see if it's ready yet (if not, then it will be polled again once the resources it needs are ready, thanks to Mio). When the Future is ready, then the function is invoked with the value yielded by the future. _Something_ has to do all that work polling futures and invoking callback functions. That's what the `Task` manages. Note that `Task` doesn't itself decide _when_ to unpark itself and poll the future, that's where Mio will end up being useful. 

## Tokio

Here's where things start getting a lot more usable. Think of Tokio as something that adapts the Futures abstractions to work with Mio to make it much easier to use. Things like network streams are now just represented as Futures, and so are operations like reading and writing to them.

### Hello World, Explained

This is just the hello world example from [the tokio repository](https://github.com/tokio-rs/tokio-core/blob/master/examples/hello.rs).

```
extern crate futures;
extern crate tokio_core;

use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

fn main() {
    let mut core = Core::new().unwrap();
    let address = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(&address, &core.handle()).unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening for connections on {}", addr);

    let clients = listener.incoming();
    let welcomes = clients.and_then(|(socket, _peer_addr)| {
        tokio_core::io::write_all(socket, b"Hello!\n")
    });
    let server = welcomes.for_each(|(_socket, _welcome)| {
        Ok(())
    });

    core.run(server).unwrap();
}
```

#### `Core`

Let's go back to the `Task` that polls a `Future` to see if it's ready yet. Something needs to tell it when to poll. A naive implementation might just continuously poll the future in a loop. Mio allows us to receive an event when the underlying resource (i.e. socket) is ready, so we could wait until then to poll the future. This is `Core`s job. 

#### `TCPListener`

Notice the `let clients = listener.incoming()` call? That returns a `Stream` of `TCPStream`s and client addresses. Tokio's `TCPListener` will automatically register with the Mio event loop, so that's all handled for you. On the next line, `clients.and_then` will invoke the given closure for each client that connects.

#### `TCPStream`

You've dealt with these before. Nothing too special, except here they have methods to `poll` the stream and register it in the Mio event loop if the stream isn't ready yet. Implements both `Read` and `Write`. Notice the call to `tokio_core::io::write_all(socket, b"Hello!\n")`, which returns a `Future` representing the completion of the write.

#### Actually making things happen

Remember when we said that Rust's Futures used a 'pull' model. Well, these things aren't just going to execute themselves. First off, there's the call to `welcomes.for_each`. This is going to continuously poll for the next item in the stream so that we keep accepting new client connections. Like their second cousins, `Iterator`s, `Stream`s are lazy. So, something needs to keep asking for the next item. `for_each` returns another `Future` whose output is `()`. If the closure passed to `for_each` ever returns an error result, then iteration is immediately stopped. This provides a way to short circuit streams.

Lastly, there's the call to `core.run(server)`. Well, all it does is just to run that one future and keep polling it until it's done.



# Tokio FizzBuzz kata

This is a bit of a twist on the normal FuzzBuzz. We're going to use Tokio to implement a simple FizzBuzz server.

The protocol is _really_ simple:

- Client simply sends a number as a string, using only the characters 0-9.
- Server responds with a single line of text:
    - 'Fizz' if the number is divisible by 3
    - 'Buzz' if the number is divisible by 5
    - 'FizzBuzz' if the number is divisible by both 3 and 5
    - If the number is not divisible by either 3 or 5, then the server responds with the number
- Bonus points if you use a Tokio client to test drive your FizzBuzz server

