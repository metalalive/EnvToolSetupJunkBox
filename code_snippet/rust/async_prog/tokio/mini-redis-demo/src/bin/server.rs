use std::mem::drop;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, broadcast};
use tokio::time::sleep;
use tokio::signal;

// why compiler does not allow to use `crate` for import ?
use mini_redis_demo::{DEFAULT_PORT, MAX_CONNECTIONS,
    Connection, cmd, SingleRequestShutdown};
use mini_redis_demo::db::FakeDatabase;

#[tokio::main]
async fn main()
{
    let url:String = format!("127.0.0.1:{}",  DEFAULT_PORT);
    if let Ok(listener) = TcpListener::bind(url).await {
        // create receiver later for each client request
        let (notify_shutdown, _) = broadcast::channel(5);
        let limit_conns = Arc::new(Semaphore::new(MAX_CONNECTIONS as usize));
        tokio::select! {
            _result = server_start(listener, &limit_conns, &notify_shutdown)
                => { println!("will never reach here"); }
            _ = signal::ctrl_c() => { println!("shutdown starts..."); }
        };
        let _ = notify_shutdown.send(());
        while limit_conns.available_permits() < (MAX_CONNECTIONS as usize)
        { sleep(Duration::new(2u64, 0)).await; }
        println!("end of testing server");
    } else {
        println!("server failed to bind port");
    }
} // end of main

async fn server_start(listener:TcpListener, limit_conns:&Arc<Semaphore>,
                      notify_shutdown:&broadcast::Sender<()> )
{
    let fakedb = FakeDatabase::new();
    // - `acquire()` and `acquire_owned()` ensures that you will get
    //   permit eventually only if semaphore is available.
    // - `acquire()` returns borrowed reference of permit instance, while
    //   `acquire_owned()` moves ownership of given Arc<Semaphore> instance, and
    //   returns a permit instance which has ownership so you can move the permit
    //   to any spawned task
    loop {
        let permit = Arc::clone(limit_conns).acquire_owned().await.unwrap();
        // the 2nd item contains IP and port of the new connection
        let (_socket, _) = listener.accept().await.unwrap();
        let fdb_cpy = fakedb.clone();
        let shutdown_monitor = notify_shutdown.subscribe();
        // a new task is spawned for each inbound socket, move the
        // socket to the new task, let Tokio runtime concurrently
        // process as many tasks as possible.
        tokio::spawn(async move {
            process_single_request(_socket, fdb_cpy, shutdown_monitor).await;
            // move the permit here, and drop it after request is done
            // processing, `drop()` returns the permit back to the semaphore
            drop(permit);
        }); // don't run it immediately by `.await`, the new task will be executed
            // in next iteration when waiting for new socket.
    } // TODO, how to break from the loop ?
} // end of server_start

async fn process_single_request (socket:TcpStream, fakedb:FakeDatabase,
                                 shutdown_monitor:broadcast::Receiver<()> )
{
    // connection allows user to read/write `redis frame` instead of
    // raw byte streams
    let mut conn = Connection::new(socket);
    let mut req_down = SingleRequestShutdown::new(shutdown_monitor) ;
    while !req_down.is_shutdown() {
        // wait on multiple concurrent branches
        // In `select!` macro block, no need to use `await` on each async expression.
        tokio::select! {
            result = conn.read_frame() => {
                let r_frm = if let Ok(Some(r)) = result {r} else {break;};
                println!("server GOT: {:?}", r_frm);
                let cmdobj:Box<dyn cmd::Command> = cmd::from_frame(r_frm).unwrap();
                // some commands may send multiple outbound frames in one go
                let _future = cmdobj.apply(&fakedb, &mut conn, &mut req_down);
                if let Err(e) = _future.await {
                    println!("[server][error] {:?}", e);
                }
            } // end of reading inbound frames
            _ = req_down.recv() => {} // will break the loop
        } // end of concurrent select
    } // end of loop
} // end of process

