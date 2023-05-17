use crate::{Connection, Frame, AsyncResult};
use crate::cmd::{
    Get, Set, Publish, Subscribe, Unsubscribe, SubscribeCommonInit,
    private_part::Command as PrivCommand
};

use async_stream::try_stream;
use bytes::Bytes;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_stream::Stream as TokioAbstractStream;
// use tracing::{debug, instrument};

pub struct Client {
    connection: Connection,
    subscribed_channels: Vec<String>,
}

pub struct Subscriber<'a> {
    client: &'a mut Client,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub channel: String,
    pub content: String,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> AsyncResult<Self> {
        // `TcpStream::connect`  performs any asynchronous DNS lookup and
        // attempts to establish the TCP connection. An error at either step
        // returns an error.
        let socket = TcpStream::connect(addr).await?;
        // Initialize the connection state. This allocates read/write buffers to
        // perform redis protocol frame parsing.
        let conn = Connection::new(socket);
        Ok( Self{connection:conn, subscribed_channels:Vec::new()} )
    }

    pub async fn get(&mut self, key: &str) -> AsyncResult<Option<Bytes>> {
        let frm = Get::new(key).into_frame();
        // Write the frame to the socket. Wait for the response from server
        self.connection.write_frame(&frm).await?;
        // Both `Simple` and `Bulk` frames are accepted. `Null` represents the
        // key not being present and `None` is returned.
        match self.read_response().await? {
            Frame::Simple(value) => Ok(Some(value.into())),
            Frame::Bulk(value) => Ok(Some(value)),
            Frame::Null => Ok(None),
            frm => Err(frm.to_error()),
        }
    }

    pub async fn set(&mut self, key: &str, value: Bytes) -> AsyncResult<()> {
        self.set_cmd(Set::new(key, value, None)).await
    }

    pub async fn set_expires(&mut self, key: &str, value: Bytes,
        expiration: Duration) -> AsyncResult<()>
    {
        self.set_cmd(Set::new(key, value, Some(expiration))).await
    }

    async fn set_cmd(&mut self, cmd: Set) -> AsyncResult<()> {
        let frm = cmd.into_frame();
        // Write the frame to the socket. Wait for the response from the server
        self.connection.write_frame(&frm).await?;
        match self.read_response().await? {
            Frame::Simple(resp) if resp == "OK" => Ok(()),
            frm => Err(frm.to_error()),
        }
    }

    pub async fn publish(&mut self, channel: &str, message: Bytes) -> AsyncResult<u64>
    {
        let frame = Publish::new(channel, message).into_frame();
        self.connection.write_frame(&frame).await?;
        match self.read_response().await? {
            Frame::Integer(response) => Ok(response),
            frame => Err(frame.to_error()),
        }
    }

    pub async fn subscribe (& mut self, channels: Vec<String>)
        -> AsyncResult<Subscriber>
    {
        // Issue the subscribe command to the server and wait for confirmation.
        // The client will then have been transitioned into the "subscriber"
        // state and may only issue pub/sub commands from that point on.
        self.subscribe_cmd(&channels).await?;
        self.subscribed_channels.extend(channels.iter().map(Clone::clone));
        Ok(Subscriber{client: self})
    }

    async fn subscribe_cmd(&mut self, channels: &[String]) -> AsyncResult<()>
    {
        let frame = Subscribe::new(channels.to_vec()).into_frame();
        self.connection.write_frame(&frame).await?;
        // check each channel the client is subscribing in the response
        for channel in channels {
            let response = self.read_response().await?;
            match response { // Verify the confirmation of subscription
                Frame::Array(ref frame) => match frame.as_slice() {
                    // The server responds with an array frame in the form of:
                    //
                    // ```
                    // [ "subscribe", channel, num-subscribed ]
                    // ```
                    //
                    // where channel is the name of the channel and
                    // num-subscribed is the number of channels that the client
                    // is currently subscribed to.
                    [subscribe, schannel, ..] // the order has to be the same
                        if *subscribe == "subscribe" && *schannel == channel => {}
                    _ => return Err(response.to_error()),
                },
                frame => return Err(frame.to_error()),
            };
        }
        Ok(())
    } // end of subscribe_cmd

    async fn read_response(&mut self) -> AsyncResult<Frame> {
        let response = self.connection.read_frame().await?;
        match response {
            Some(Frame::Error(msg)) => Err(msg.into()),
            Some(frame) => Ok(frame),
            None => {
                // Receiving `None` here indicates the server has closed the
                // connection without sending a frame. This is unexpected and is
                // represented as a "connection reset by peer" error.
                let err = Error::new(ErrorKind::ConnectionReset, "connection reset by server");
                Err(err.into())
            }
        }
    }
    
    pub fn get_subscribed(&self) -> &[String] {
        &self.subscribed_channels
    } // Returns the set of channels currently subscribed to.

    pub async fn unsubscribe(&mut self, channels: &[String]) -> AsyncResult<()> {
        let frame = Unsubscribe::new(channels.to_vec()).into_frame();
        self.connection.write_frame(&frame).await?;
        // if the input channel list is empty, server acknowledges as unsubscribing
        // from all subscribed channels, so we assert that the unsubscribe list received
        // matches the client subscribed one
        let num = if channels.is_empty() {
            self.subscribed_channels.len()
        } else {
            channels.len()
        };
        /*
        for _ in 0..num { // Read the response
            let response = self.client.read_response().await?;
            match response {
                Frame::Array(ref frame) => match frame.as_slice() {
                    [unsubscribe, channel, ..] if *unsubscribe == "unsubscribe" => {
                        let len = self.subscribed_channels.len();
                        if len == 0 {
                            // There must be at least one channel
                            return Err(response.to_error());
                        }
                        // unsubscribed channel should exist in the subscribed list at this point
                        self.subscribed_channels.retain(|c| *channel != &c[..]);
                        // Only a single channel should be removed from the
                        // list of subscribed channels.
                        if self.subscribed_channels.len() != len - 1 {
                            return Err(response.to_error());
                        }
                    }
                    _ => return Err(response.to_error()),
                },
                frame => return Err(frame.to_error()),
            };
        }
         * */
        Ok(())
    } // end of unsubscribe
} // end of impl Client


impl<'a> Subscriber<'a> {
    // Receive the next message published on a subscribed channel, waiting if
    // necessary.
    // `None` indicates the subscription has been terminated.
    async fn next_message(&mut self) -> AsyncResult<Option<Message>> {
        match self.client.connection.read_frame().await? {
            Some(mframe) => match mframe {
                Frame::Array(ref frm) => match frm.as_slice() {
                    [ftyp, chn, content] if *ftyp == "message" =>
                        Ok(Some(Message {  channel: chn.to_string(),
                            content: content.to_string(),
                        })),
                    _ => Err(mframe.to_error()),
                }, // destruct a received frame to 3 pieces, check whether it
                   // contains published message with a channel.
                   // See `Subscribe::make_message_response()` in `src/cmd/subscribe.rs`
                   // to understand how the message frame is formed.
                frm => Err(frm.to_error()),
            },
            None => Ok(None),
        }
    } // end of  next_message

    pub fn into_stream(&'a mut self) ->
        impl TokioAbstractStream<Item=AsyncResult<Message>> + 'a
    { // `self`, `self.client` and the output, must have the same lifetime
      // , otherwisse compiler will report E0700 : hidden type for `impl xxx`
      // captures lifetime that does not appear in bounds
        try_stream! {
            while let Some(msg) = self.next_message().await? {
                yield msg;
            }
        } // note the crate `async_stream` requires unstable feature `yield`
          // for generating sequence of message frames.
    }
} // end of impl Subscriber
