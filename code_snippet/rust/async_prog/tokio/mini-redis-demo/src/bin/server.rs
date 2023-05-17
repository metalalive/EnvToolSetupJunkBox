use tokio::net::{TcpListener, TcpStream};
use tokio::signal;

// why compiler does not allow to use `crate` for import ?
use mini_redis_demo::{DEFAULT_PORT, Connection, cmd};
use mini_redis_demo::db::FakeDatabase;

#[tokio::main]
async fn main()
{
    let url:String = format!("127.0.0.1:{}",  DEFAULT_PORT);
    let future = TcpListener::bind(url);
    if let Ok(listener) = future.await {
        let fakedb = FakeDatabase::new();
        tokio::select! {
            _result = server_start(listener, &fakedb) => {
                println!("will never reach here");
            }
            _ = signal::ctrl_c() => {
                println!("shutdown starts...");
            } // TODO, extra channel for each ongoing task to receive shutdown signal
        };
    } else {
        println!("server failed to bind port");
    }
} // end of main

async fn server_start(listener:TcpListener, fakedb:&FakeDatabase)
{
    loop {
        // the 2nd item contains IP and port of the new connection
        let (_socket, _) = listener.accept().await.unwrap();
        let fdb_cpy = fakedb.clone();
        // a new task is spawned for each inbound socket, move the
        // socket to the new task, let Tokio runtime concurrently
        // process as many tasks as possible.
        tokio::spawn(async move {
            process(_socket, fdb_cpy).await;
        });
    }
}

async fn process(socket:TcpStream, fakedb:FakeDatabase)
{
    // connection allows user to read/write `redis frame` instead of
    // raw byte streams
    let mut conn = Connection::new(socket);
    while let Some(r_frm) = conn.read_frame().await.unwrap()
    {
        println!("server GOT: {:?}", r_frm);
        let cmdobj:Box<dyn cmd::Command> = cmd::from_frame(r_frm).unwrap();
        // some commands may send multiple outbound frames in one go
        let _future = cmdobj.apply(&fakedb, &mut conn);
        if let Err(e) = _future.await {
            println!("[server][error] {:?}", e);
        }
    } // end of loop for reading inbound frames
} // end of process

