use bytes::Bytes;
use async_trait::async_trait;

use crate::{Connection, AsyncResult, Parse, Frame};
use crate::db::FakeDatabase;
use crate::cmd::{Command as PubCommand, private_part::Command as PrivCommand};

#[derive(Debug)]
pub struct Get {
    key: String,  // Name of the key to get
}

impl Get {
    pub fn new(k: impl ToString) -> Self {
        Self {key: k.to_string(),}
    }
    pub fn key(&self) -> &str { &self.key }
}

#[async_trait]
impl PubCommand for Get {
    // Apply the `Get` command to the specified `Db` instance.
    // The response is written to `dst`. This is called by the server in order
    // to execute a received command.
    async fn apply(&self, fdb: &FakeDatabase, dst: &mut Connection)
        -> AsyncResult<()>
    {
        let response = match fdb.get(self.key()) {
            Ok(v) => {
                // `Frame::Bulk` expects data to be of type `Bytes`,
                // `w` is `&Vec<u8>` type, converted to `Bytes` using `into()`.
                if let Some(w) = v {
                    Frame::Bulk(w.into())
                } else { Frame::Null }
            },
            Err(e) => {
                // should return error frame, if it is lock failure, specific number
                // of times to retry (TODO)
                Frame::Error(e.to_string()) 
            }
        };
        dst.write_frame(&response).await?;
        Ok(())
    }
} // end of Get command

impl PrivCommand for Get {
    // Parse a `Get` instance from a received frame.
    // The `Parse` argument provides a cursor-like API to read fields from the
    // `Frame`. At this point, the entire frame has already been received from
    // the socket.
    //
    // # Format
    // Expects an array frame containing two entries.
    // ```text
    // GET key
    // ```
    fn parse_frames(parse: &mut Parse) -> AsyncResult<Box<dyn PubCommand>> {
        // The `GET` string has already been consumed. The next value is the
        // name of the key to get. If the next value is not a string or the
        // input is fully consumed, then an error is returned.
        let key = parse.next_string()?;
        let obj = Get{key};
        Ok(Box::new(obj)) 
    }

    // Converts the command into an equivalent `Frame`.
    // This is called by the client when encoding a `Get` command to send to
    // the server.
    fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("get".as_bytes()));
        frame.push_bulk(Bytes::from(self.key.into_bytes()));
        frame
    }
} // end of Get command
