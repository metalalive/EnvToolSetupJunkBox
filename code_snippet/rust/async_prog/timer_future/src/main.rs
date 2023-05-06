use std::{
    future::Future,
    time::Duration,
    sync::{Arc, Mutex},
    // this async example runs under single-thread scenario, I use
    // `sync-channel` instead of  `channel`
    sync::mpsc::{sync_channel, SyncSender, Receiver},
    task::Context
};

use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake}
};

use rand::Rng;
use timer_future::TimerFuture;

type AsynReturnType = ();

static QFULL_ERR_MSG:& str = "failed to send more task, queue is full";

// executor receives tasks off the channel and runs them
struct Executor {
    rdy_q : Receiver<Arc<Task>>,
}

// Spawn new `task` instance and send them to the channel.
// Note task is wrapped with `Arc` pointer
#[derive(Clone)]
struct Spawner {
    send_q: SyncSender<Arc<Task>>,
}

// a task which can reschedule itself to be pulled by an executor
struct Task {
    // `Mutex` is not necessary in single-threaded scenario, it is for
    // proof of thread-safety to Rust compiler.
    future:Mutex<Option<BoxFuture<'static, AsynReturnType>>>,
    // handle to put itself back to the sender channel again
    send_q: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn(&self, _future:impl Future<Output=AsynReturnType> + Send + 'static)
    {
        // `futures` crate seems to do the magic automatically which converts
        // `Future` trait in standard library to `FutureExt` trait, so the variable
        // `_future` here can invoke `boxed()` which wraps the `Future` instance
        // in a `Box` then `Pin` it.
        let _future = _future.boxed();
        // the possible alternative might be Box::pin(_future)
        let t = Task {
            // remind it is acceptable to have multiple senders 
            send_q: self.send_q.clone(),
            future: Mutex::new(Some(_future))
        };
        let t = Arc::new(t);
        self.send_q.send(t).expect(QFULL_ERR_MSG);
    }
} // end of Spawner

impl ArcWake for Task {
    fn wake_by_ref(aself:&Arc<Self>) {
        // implement `wake()` by sending the same task back
        // to the channel, and cloning new Arc to the task
        let cloned = Arc::clone(&aself); // alternatively, aself.clone();
        aself.send_q.send(cloned).expect(QFULL_ERR_MSG) ;
    }
}

impl Executor {
    fn run(&self) {
        // remind `recv()` is a blocking call
        while let Ok(_tsk) = self.rdy_q.recv() {
            let _tsk_typtest : & Arc<Task> = &_tsk;
            if let Ok(mut s) = _tsk.future.lock() {
                if let Some(mut fut) = s.take() {
                    // generate `waker` instance using reference to `arc`
                    // counter pointed to the task
                    let wakr = waker_ref(&_tsk);
                    let ctx = & mut Context::from_waker(&wakr);
                    // why `as_mut()` at here ?
                    if fut.as_mut().poll(ctx).is_pending() {
                        *s = Some(fut);
                    }
                } else {
                    println!("future object not found");
                }
            } else {
                println!("lock failure in executor.run");
            }
        } // end of loop, will break when all senders to the same
          // channel are disposed
    }
} // end of Executor

fn init_executor_and_spawner() -> (Executor, Spawner)
{
    const MAX_QUEUED_TASKS:usize = 1000;
    let (tx, rx) = sync_channel(MAX_QUEUED_TASKS);
    (Executor{rdy_q:rx}, Spawner{send_q:tx})
}

async fn _asyn_demo_start (msgs:Vec<&str>)
{ // to call this function directly, it will return `Future` instance
    let mut iterobj = msgs.iter();
    if let Some(s0) = iterobj.next() {
        println!("{}", s0);
    }
    for s in iterobj {
        let dly_sec:u8 = rand::thread_rng().gen_range(1..=5);
        TimerFuture::new(Duration::new(dly_sec as u64, 0)).await;
        println!("{}", s);
    }
}

fn main() {
    let (exec, spawner)  = init_executor_and_spawner();
    let msgs1 = vec!["careful code", "clean pants", "easy extension"];
    let msgs2 = vec!["dirty code", "poopy pants"];
    spawner.spawn(_asyn_demo_start(msgs1));
    spawner.spawn(_asyn_demo_start(msgs2));
    std::mem::drop(spawner);
    // invalidate the variable and dispose the instance before executor
    exec.run();
    println!("--- end of custom future / executor demo ---");
}

