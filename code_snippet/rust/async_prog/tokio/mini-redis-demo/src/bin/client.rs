use std::time::Duration;
use std::collections::{HashMap};

// The crate `Bytes` features shallow copy of raw bytes in
// network transferring
use bytes::Bytes;
use tokio::time::sleep;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::StreamExt as TokioStreamExt;

use mini_redis_demo::{DEFAULT_PORT, AsyncResult};
use mini_redis_demo::clients::{Message as SubsMessage, Client};

type Responder<T> = oneshot::Sender<AsyncResult<T>>;

#[derive(Debug)]
enum KeyValCommand {
    GET {resp:Responder<Option<Bytes>>, key:String, },
    SET {resp:Responder<()>, key:String, val:Bytes},
}
async fn kv_exp_sender(tx:&mpsc::Sender<KeyValCommand>, k:&str, unchk_val:Option<&str>)
    -> Option<String>
{
    let k = k.to_string();
    if let Some(v) = unchk_val {
        let (resp_tx, _resp_rx) = oneshot::channel();
        let cmd = KeyValCommand::SET {
            key:k, resp:resp_tx, val:Bytes::from(v.to_string()), // v.into()
        };
        let _ = tx.send(cmd).await;
        Some(String::from("ok"))
    } else { // the `resp_tx` has different sizes for GET/SET command
             // they have to be declared in distinct scopes to let compiler
             // understand that it is safe to go
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = KeyValCommand::GET {key:k, resp:resp_tx};
        let _ = tx.send(cmd).await;
        // receiver of oneshot channel doesn't have `recv()`, simply call
        // `await` to get result
        if let Ok(result) = resp_rx.await {
            let result:Option<Bytes> = result.unwrap();
            if let Some(rawbytes) = result {
                // in this example I only insert printable bytes, all bytes read
                // from the client connection can be safely converted to ASCII
                // characters
                Some(rawbytes.escape_ascii().to_string())
            } else {
                None
            }
        } else {
            println!("error from oneshot receiver");
            None
        }
    }
} // end of kv_exp_sender

async fn kv_exp_receiver(mut rx:mpsc::Receiver<KeyValCommand>)
{
    let url:String = format!("127.0.0.1:{}",  DEFAULT_PORT);
    let mut myc = Client::connect(url).await.unwrap();
    while let Some(cmd) = rx.recv().await {
        match cmd {
            KeyValCommand::GET{key,resp} => {
                let result = myc.get(&key).await;
                let _ = resp.send(result);
            },
            KeyValCommand::SET{val,key,resp} => {
                let result = myc.set(&key, val).await;
                let _ = resp.send(result);
            },
            // 1. let command senders handle their errors if it exists in the result
            // 2. sender of oneshot channel doesn't need `await` cuz it fails or
            //    succeeds immediately after the call to `send(...)`
        };
    } // end of loop
} // end of kv_exp_receiver

// macro below expands the function below to synchronous main
// function calling asynchronous main function.
#[tokio::main]
async fn main() ->  AsyncResult<()> {
    let (tx, rx) = mpsc::channel(29);
    let tx2 = tx.clone();
    let manager = tokio::spawn(async move {
        kv_exp_receiver(rx).await;
    });
    let tsk1 = tokio::spawn(async move {
        let _tx = tx;
        let _ = kv_exp_sender(&_tx, "halo", Some("code ocean")).await;
        let result = kv_exp_sender(&_tx, "halo", None).await;
        assert_eq!(result.unwrap().as_str(), "code ocean");
        let result = kv_exp_sender(&_tx, "not-exists", None).await;
        assert_eq!(result, None);
    });
    let tsk2 = tokio::spawn(async move {
        let _tx = tx2;
        kv_exp_sender(&_tx, "mineral", Some("electricity")).await;
        kv_exp_sender(&_tx, "museum", Some("gallery")).await;
        let result = kv_exp_sender(&_tx, "mineral", None).await;
        assert_eq!(result.unwrap().as_str(), "electricity");
        kv_exp_sender(&_tx, "halo", Some("in-depth knowledge")).await;
        let result = kv_exp_sender(&_tx, "halo", None).await;
        assert_eq!(result.unwrap().as_str(), "in-depth knowledge");
    });
    manager.await.unwrap(); // check the result by trying to unwrap the `result`
    tsk1.await.unwrap();
    tsk2.await.unwrap();
    { // reconnect
        let url:String = format!("127.0.0.1:{}",  DEFAULT_PORT);
        let mut myc = Client::connect(url).await?;
        let result = myc.get("not-exists").await?;
        assert_eq!(result, None);
        let result = myc.get("halo").await?;
        assert_eq!(result, Some(Bytes::from("in-depth knowledge")));
        let result = myc.get("mineral").await?;
        assert_eq!(result, Some(Bytes::from("electricity")));
        myc.set("halo", "blooming".into()).await?;
        myc.set("rdbms", "redo log".into()).await?;
        let result = myc.get("halo").await?;
        assert_eq!(result, Some(Bytes::from("blooming")));
    }
    let publisher1 = tokio::spawn(async {
        let pairs = vec![("halo", "Satu"), ("halo", "deux"), ("halo", "san"),
            ("halo", "four"), ];
        let _ = _run_publish_messages(pairs, 1).await;
    });
    let publisher2 = tokio::spawn(async {
        let pairs = vec![("food", "dumpling"), ("food", "Pad Kra Pao"), ("food", "pita"),
            ("food", "goat stew"), ("food", "soba"), ("food", "Burger"), ];
        let _ = _run_publish_messages(pairs, 1).await;
    });
    let subscriber1 = tokio::spawn(async {
        let url:String = format!("127.0.0.1:{}",  DEFAULT_PORT);
        let mut myc = Client::connect(url).await .unwrap();
        let mut expect_recv_data: HashMap<String, Vec<String>> = HashMap::new();
        let data = vec![
            "dumpling".to_string(),     "Pad Kra Pao".to_string(),
            "pita".to_string(),         "goat stew".to_string(),
            "soba".to_string(),         "Burger".to_string()
        ];
        expect_recv_data.insert("food".to_string(), data);
        let _ = _run_subscription(&expect_recv_data, &mut myc).await;
    });
    let subscriber2 = tokio::spawn(async{
        let url:String = format!("127.0.0.1:{}",  DEFAULT_PORT);
        let mut myc = Client::connect(url).await .unwrap();
        let mut expect_recv_data: HashMap<String, Vec<String>> = HashMap::new();
        let data = vec![
            "dumpling".to_string(), "Pad Kra Pao".to_string(),
            "pita".to_string(),     "goat stew".to_string(),
            "soba".to_string(),     "Burger".to_string()
        ];
        expect_recv_data.insert("food".to_string(), data);
        let data = vec![
            "Satu".to_string(), "deux".to_string(),
            "san".to_string(),  "four".to_string(),
        ];
        expect_recv_data.insert("halo".to_string(), data);
        let _ = _run_subscription(&expect_recv_data, &mut myc).await;
    });
    publisher1.await.unwrap();
    publisher2.await.unwrap();
    subscriber1.await.unwrap();
    subscriber2.await.unwrap();
    let subscriber3 = tokio::spawn(async {
        let url:String = format!("127.0.0.1:{}",  DEFAULT_PORT);
        let mut myc = Client::connect(url).await .unwrap();
        let mut expect_recv_data: HashMap<String, Vec<String>> = HashMap::new();
        let data = vec!["Kofta".to_string(), "Lepet".to_string()];
        expect_recv_data.insert("food".to_string(), data);
        let _ = _run_subscription(&expect_recv_data, &mut myc).await;
        let data:&mut Vec<String> = expect_recv_data.get_mut("food").unwrap();
        data.clear();
        data.extend( vec!["meat pie".to_string(), "taco".to_string()]);
        let data = vec![ "hive-five".to_string(), "a789".to_string(),
            "0xbeef".to_string() ];
        expect_recv_data.insert("halo".to_string(), data);
        let _ = _run_subscription(&expect_recv_data, &mut myc).await;
        let data = ["halo".to_string()];
        let _ = myc.unsubscribe(&data).await;
    });
    let publisher3 = tokio::spawn(async {
        let pairs = vec![("food", "Kofta"), ("halo", "Ichi"), ("food", "Lepet")];
        let _ = _run_publish_messages(pairs, 1).await;
        let pairs = vec![("halo", "hive-five"), ("food", "meat pie"),
            ("halo", "a789"), ("food", "taco"), ("halo", "0xbeef"), ];
        let _ = _run_publish_messages(pairs, 2).await;
    });
    subscriber3.await.unwrap();
    publisher3.await.unwrap();
    println!("end of mini-redis client test");
    Ok(())
} // end of main


async fn _run_publish_messages(pairs:Vec<(&str, &str)>, dly:u16)
    -> AsyncResult<()>
{ // tokio executor will switch to another task while current
  // publisher is sleeping
    sleep(Duration::new(dly as u64, 0)).await;
    let url:String = format!("127.0.0.1:{}",  DEFAULT_PORT);
    let mut myc = Client::connect(url).await?;
    for (chn,msg) in pairs {
        let bmsg = Bytes::from(msg.to_string());
        let result = myc.publish(chn, bmsg).await;
        if let Err(e) = result {
            println!("channel:{}, msg:{}, error:{:?}", chn, msg, e);
        }
    }
    Ok(())
} // end of _run_publish_messages

async fn _run_subscription (expect_recv_data:&HashMap<String, Vec<String>>,
                            myc:&mut Client) -> AsyncResult<()>
{
    let mut actual_recv_data: HashMap<String, Vec<String>> = HashMap::new();
    let mut subscribing_channels:Vec<String> = vec![];
    let mut expect_num_recv_msgs = 0;
    for (k,v) in expect_recv_data.iter() {
        expect_num_recv_msgs += v.len();
        actual_recv_data.insert(k.clone(), Vec::new());
        subscribing_channels.push(k.clone());
    }
    let mut onesubs = myc.subscribe(subscribing_channels).await .unwrap();
    // the stream object below is implemented using `TokioStreamExt` trait
    let _stream  = onesubs.into_stream();
    // the syntax `impl Trait` is allowed ONLY in function parameters, not closure
    let discard_empty = |obj:&AsyncResult<SubsMessage>|-> bool {
        match obj {
            Ok(m)  => (m.content.len() > 0) && (m.channel.len() > 0),
            _others => false,
        }
    };
    let processpipe = _stream.filter(discard_empty).take(expect_num_recv_msgs);
    tokio::pin!(processpipe);
    while let Some(msg) = processpipe.next().await {
        let msg = msg.unwrap();
        if let Some(elms) = actual_recv_data.get_mut(&msg.channel) {
            elms.push(msg.content);
        } else {
            println!("something wrong, receive message not subscribed {}"
                     , msg.channel);
        }
    }
    for k in expect_recv_data.keys() {
        let expect_seq = expect_recv_data.get(k).unwrap();
        let actual_seq = actual_recv_data.get(k).unwrap();
        assert_eq!(expect_seq, actual_seq);
    }
    Ok(())
} // end of _run_subscription

