
Mini-Redis for learning Tokio crate

#### Supported commands
- get value by key
- set key / value pair
- publish message with specific channel
- subsribe / unsubscribe to specific channel, then receive streaming messages

#### Build
```
cargo build --bin server
cargo build --bin client
```

#### Run
```
cargo build --bin server
cargo build --bin client
```

Alternatively, run client/server programs with valgrind check :
```
valgrind ./target/debug/server
valgrind ./target/debug/client
```

The client program will hang, this is for testing graceful shutdown.
You can terminate the server by `ctrl + c` then client will proceed to the end normally.

#### Issues
- memory possibly lost in global signal handler of Tokio crate, [#4756](https://github.com/tokio-rs/tokio/issues/4756)

