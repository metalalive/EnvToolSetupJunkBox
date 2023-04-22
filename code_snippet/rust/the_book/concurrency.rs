use std::thread;
use std::sync::{mpsc, Mutex, Arc};
use std::time::Duration;

fn  simultaneous_run_demo()
{
    println!("-------- simultaneous run demo --------");
    let txq = vec![8.71, -22.56];
    let rxq = vec!["bible", "kuran", "pali canon"];
    let t_hdlr_1 = thread::spawn(move || {
        println!("queue for TX {:?}", txq);
        for idx in 1..10 {
            thread::sleep(Duration::from_millis(1));
            println!("thread 2 running, idx {idx}");
        }
    });
    let t_hdlr_2 = thread::spawn(move || {
        thread::sleep(Duration::from_millis(3));
        println!("queue for RX {:?}", rxq);
    });
    for jdx in 1..4 {
        thread::sleep(Duration::from_millis(2));
        println!("main thread running, jdx {jdx}");
    }
    t_hdlr_1.join().unwrap();
    t_hdlr_2.join().unwrap();
} // end of simultaneous_run_demo

fn msg_passing_demo()
{
    println!("-------- message passing demo --------");
    let (tx, rx) = mpsc::channel();
    let txcpy = tx.clone();
    let t_hdlr_1 = thread::spawn(move || {
        let data = vec!["Guam", "Indonesia", "Malaysia", "Palau"];
        for s in data {
            thread::sleep(Duration::from_millis(100));
            tx.send(s).unwrap();
        }
    });
    let t_hdlr_2 = thread::spawn(move || {
        let data = vec!["GMT", "jar", "crate"];
        for s in data {
            thread::sleep(Duration::from_millis(99));
            txcpy.send(s).unwrap();
        }
    });
    let t_hdlr_3 = thread::spawn(move || {
        println!("receiver started");
        for _ in 0..7 {
            let s:&str = rx.recv().unwrap(); // recv() is a blocking method
            println!("data received ... {s}");
        }
    });
    t_hdlr_1.join().unwrap();
    t_hdlr_2.join().unwrap();
    t_hdlr_3.join().unwrap();
} // end of msg_passing_demo


fn multithread_mutex_demo()
{
    println!("-------- multithread mutex demo --------");
    let counter = Arc::new(Mutex::new(0));
    let mut hdlrs = vec![];
    let num_threads = 10;
    for _ in 0..num_threads {
        let cnt = Arc::clone(&counter);
        let hdl = thread::spawn(move || {
            let mut c = cnt .lock() .unwrap();
            println!("current counter : {c}");
            *c += 3;
        });
        hdlrs.push(hdl);
    }
    for hdl in hdlrs {
        hdl.join().unwrap();
    }
    let finalcnt = counter .lock() .unwrap();
    println!("final counter : {finalcnt}");
} // end of multithread_mutex_demo

fn main() {
    simultaneous_run_demo();
    msg_passing_demo();
    multithread_mutex_demo();
} // end of main
