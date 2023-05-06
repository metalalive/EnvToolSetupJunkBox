use std::{
    thread,
    future::Future,
    pin::Pin,
    time::Duration,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker}
};

pub struct TimerFuture {
    // the shared state at here may come from different tasks in
    // different threads,  it should be protected against mutex
    // and counter under multithreaded scene.
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    // whether or not the sleep time has been elapsed
    completed:bool,
    // waker for the task `TimerFuture` is running on.
    // the thread can tell `TimerFuture` to wake up using
    // the `waker` after `completed` flag is set
    waker:Option<Waker>,
    num_retry:u32,
}

impl Future for TimerFuture
{
    type Output = String;
    // which lifetime should I annotate to context object ?
    fn poll(self:Pin<&mut Self>, cx:& mut Context<'_>) -> Poll<Self::Output>
    {
        let lockresult = self.shared_state.lock();
        match lockresult {
            Ok(mut s) => {
                if s.completed {
                    Poll::Ready("job done".to_string())
                } else {
                    // keep the weaker from context, is it necessary to clone
                    // it in more advanced use cases ?
                    s.waker = Some(cx.waker().clone());
                    s.num_retry += 1;
                    Poll::Pending
                }
            },
            Err(_) => {
                Poll::Ready("lock failure".to_string())
            },
        }
    }
} // end of TimerFuture

impl Drop for SharedState
{
    fn drop(&mut self) {
        println!("shared state, num of retries, {}", self.num_retry);
    }
}

impl TimerFuture
{
    pub fn new(seconds:Duration) -> Self {
        let state = SharedState{waker:None, completed:false
            , num_retry:0u32 };
        let state = Arc::new(Mutex::new(state));
        let statecpy = Arc::clone(&state);
        // the 2 threads will modify the state later
        thread::spawn(move || {
            _run_timeout_event(statecpy, seconds);
        });
        Self{shared_state:state}
    }
} // end of TimerFuture

fn _run_timeout_event(state:Arc<Mutex<SharedState>>, seconds:Duration)
{ // assume this TimerFuture object will be woken up after few seconds
    thread::sleep(seconds);
    let lockresult = state.lock();
    match lockresult {
        Ok(mut s) => {
            s.completed = true;
            if let Some(wk) = s.waker.take() {
                wk.wake();
            } // wake() is called only at here
        },
        Err(e) => {
           println!("lock failure in another thread, {:?}", e);
        },
    }
}

