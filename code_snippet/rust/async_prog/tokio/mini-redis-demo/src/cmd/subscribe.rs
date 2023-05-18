use std::pin::Pin;
use std::thread::sleep;
use std::time::Duration;
use std::cell::RefCell;

use bytes::Bytes;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tokio_stream::{Stream as TokioAbstractStream, StreamExt, StreamMap};

use crate::{Connection, AsyncResult, Parse, ParseError, Frame, SingleRequestShutdown};
use crate::cmd::{Command as PubCommand, private_part::Command as PrivCommand};
use crate::db::FakeDatabase;

// the trait `Stream` in `tokio-stream` doesn't implement `Send` trait
// , but it is required for transferring ownership of new messages, so
// add it in application code.
type MessagesPipe = Pin<Box<dyn TokioAbstractStream<Item = Bytes> + Send>>;

#[derive(Debug)]
pub struct Subscribe {
    // this project requires interior mutability on the field,
    // also it is accessible only by the thread running the `Subscribe` instance.
    channels:RefCell<Vec<String>>,
}
// This project guarantees that a `Subscribe` instance won't be referenced
// and mutated simultaneously among several threads. It should be harmless
// to implement the traits `Sync`
unsafe impl Sync for Subscribe {}

#[derive(Debug)]
pub struct Unsubscribe {
    channels: Vec<String>,
}

pub trait CommonInit {
    fn new(chns:Vec<String>) -> Self;
}
impl CommonInit for Subscribe {
    fn new(chns:Vec<String>) -> Self {
        Self{channels: RefCell::new(chns)}
    }
}
impl CommonInit for Unsubscribe {
    fn new(chns:Vec<String>) -> Self {
        Self{channels: chns}
    }
}

// - each individual channel is handled using a `tokio::sync::broadcast::Receiver`
//   channel, messages are then fanned out to all clients that already subscribed
//   the channel in advance.
// - Note an individual client may subscribe / unsubscribe multiple channels at
//   any given time.
#[async_trait]
impl PubCommand for Subscribe {
    async fn apply(&self, db:&FakeDatabase, dst:&mut Connection,
                   shutdown:&mut SingleRequestShutdown) -> AsyncResult<()>
    { // gather streams for all channels, has to be declared as mutable instance
        let mut subscriptions:StreamMap<String, MessagesPipe> = StreamMap::new();
        while !shutdown.is_shutdown()  {
            let _chns:Vec<String> = {
                // the inner scope ensures the mutable reference from `RefMut<T>` type
                // will be dropped earlier before calling the async functions
                let x = & mut * self.channels.borrow_mut();
                x.drain(..).collect()
            };
            for chn in _chns {
                let num_subs = if subscriptions.contains_key(chn.as_str()) {
                        subscriptions.len()
                    } else {
                        refresh_subscriptions(&mut subscriptions, db, chn.clone())
                    };
                let frm = self.make_subscribe_response(chn, num_subs);
                dst.write_frame(&frm).await ?;
            } // move the new channel labels to given stream map
            tokio::select! {
                // Note the method `next()` comes from the trait `StreamExt`
                Some((k, v)) = subscriptions.next() => {
                    let frm = self.make_message_response(k,v);
                    dst.write_frame(&frm).await?;
                }
                result = dst.read_frame() => {
                    let result = match result {Ok(r) => r, _others => break}; // network error
                    let frm = match result {Some(f) => f, None => break}; // end of stream
                    let result = self.handle_cmd_in_stream(frm, &mut subscriptions)?;
                    if let Some(frm) = result {
                        dst.write_frame(&frm).await?;
                    }
                } // more frames from client
                _ = shutdown.recv() => {
                    println!("receive shutdown when streaming to subcribers");
                } // will break the loop
            }; // end of macro tokio::select
        } // end of loop
        Ok(())
    } // end of apply
} // end of impl PubCommand

impl PrivCommand for Subscribe {
    fn parse_frames(parse: &mut Parse) -> AsyncResult<Box<dyn PubCommand>>
    {
        match  inner_parse_frames::<Self>(parse) {
            Ok(v) => Ok(Box::new(v)),
            Err(e) => Err(e),
        }
    }
    fn into_frame(self) -> Frame
    {
        let mut frm = Frame::array();
        frm.push_bulk(Bytes::from("subscribe".as_bytes()));
        let chns = self.channels.borrow();
        for c in &*chns { // only require the reference to each string
            let cb = Bytes::from(c.clone().into_bytes());
            frm.push_bulk(cb);
        }
        frm
    }
} // end of impl PrivCommand

impl PrivCommand for Unsubscribe {
    fn into_frame(self) -> Frame
    {
        let mut frm = Frame::array();
        frm.push_bulk(Bytes::from("unsubscribe".as_bytes()));
        for c in self.channels {
            let cb = Bytes::from(c.into_bytes());
            frm.push_bulk(cb);
        }
        frm
    }
} // end of impl PrivCommand


impl Subscribe {
    fn make_subscribe_response(&self, channel:String, num_subs:usize) -> Frame
    {
        let mut frm = Frame::array();
        frm.push_bulk(Bytes::from_static(b"subscribe"));
        frm.push_bulk(Bytes::from(channel));
        frm.push_int(num_subs as u64);
        frm
    }
    fn make_message_response(&self, chn:String, msg:Bytes) -> Frame
    {
        let mut frm = Frame::array();
        frm.push_bulk(Bytes::from_static(b"message"));
        frm.push_bulk(Bytes::from(chn));
        frm.push_bulk(msg);
        frm
    }
    fn handle_cmd_in_stream(&self, frm:Frame, subscriptions:&mut StreamMap<String, MessagesPipe>
                           ) -> AsyncResult<Option<Frame>>
    { // commands received in the middleware of streaming process
        let mut parsed = Parse::new(frm)?;
        let command_name = parsed.next_string()?.to_lowercase();
        let out = match &command_name[..] {
            "subscribe" => {
                let cmd2 = inner_parse_frames::<Self>(&mut parsed)?;
                println!("streaming server GOT: {:?}", cmd2);
                let dst:&mut Vec<String> = & mut * self.channels.borrow_mut();
                let src:&mut Vec<String> = & mut * cmd2.channels.borrow_mut();
                let src:Vec<String> = src.drain(..).collect();
                dst.extend(src);
                None
            },
            "unsubscribe" => {
                let mut cmd2 = inner_parse_frames::<Unsubscribe>(&mut parsed)?;
                println!("streaming server GOT: {:?}", cmd2);
                if cmd2.channels.is_empty() {
                    let src:Vec<String> = subscriptions.keys()
                        .map(|k| k.to_string()).collect();
                    cmd2.channels.extend(src);
                }
                for chn in &cmd2.channels {
                    subscriptions.remove(chn);
                }
                let frm = cmd2.make_response(subscriptions.len()) ;
                Some(frm)
            },
            _others => {
                let detail = format!("not supported in stream, type:{}",
                                     &command_name[..] );
                Some(Frame::Error(detail))
            }
        };
        parsed.finish()?;
        Ok(out)
    } // end of handle_cmd_in_stream
} // end of Subscribe

impl Unsubscribe {
    fn make_response(&self, num_subs:usize) -> Frame
    {
        let mut frm = Frame::array();
        frm.push_bulk(Bytes::from_static(b"unsubscribe"));
        // frm.push_bulk(Bytes::from(channel));
        frm.push_int(num_subs as u64);
        frm
    }
}


fn inner_parse_frames<T:CommonInit>(parse: &mut Parse) -> AsyncResult<T>
{
    let mut _channels = vec![];
    loop {
        match parse.next_string() {
            Ok(chn) => { _channels.push(chn); },
            Err(ParseError::EndOfStream) => break,
            Err(e) => return Err(e.into()),
        };
    };
    Ok(T::new(_channels))
}

// internal work for getting broadcast receiver, responding to client with
// subscription result, generate and register a stream.
fn refresh_subscriptions (subscriptions:&mut StreamMap<String, MessagesPipe>,
                                db:&FakeDatabase, channel:String) -> usize
{
    let mut rx = loop {
        if let Ok(v) = db.subscribe(channel.clone()) {
            break v 
        } else {
            sleep(Duration::new(1u64, 0));
        } // retry until success
    };
    let streaming_rx = async_stream::stream!{
        loop {
            match rx.recv().await {
                Ok(msg) => yield msg,
                Err(broadcast::error::RecvError::Lagged(_)) => {},
                Err(_) => break,
            }
        }
    };
    let streaming_src = Box::pin(streaming_rx);
    subscriptions.insert(channel.clone(), streaming_src);
    subscriptions.len()
} // end of  refresh_subscriptions

