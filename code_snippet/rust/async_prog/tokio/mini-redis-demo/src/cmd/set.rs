use std::time::Duration;
use bytes::Bytes;
use async_trait::async_trait;

use crate::{Connection, AsyncResult, Parse, ParseError, Frame};
use crate::cmd::{Command as PubCommand, private_part::Command as PrivCommand};
use crate::db::FakeDatabase;

#[derive(Debug)]
pub struct Set {
    key: String,  // Name of the key to get
    value: Bytes,
    expire: Option<Duration>,
}

impl Set {
    pub fn new(k:impl ToString, v:Bytes, exp: Option<Duration>) -> Self {
        Self {key: k.to_string(), value:v, expire:exp}
    }
    pub fn key(&self) -> &str { &self.key }
    pub fn value(&self) -> &Bytes { &self.value }
}

#[async_trait]
impl PubCommand for Set {
    async fn apply(&self, fdb: &FakeDatabase, dst: &mut Connection)
        -> AsyncResult<()>
    {
        // may require error handling once it goes huge
        // , the `value()` returns `Bytes`, require 3rd-party crate `bytes`
        let _ = fdb.set(self.key(), self.value().to_vec());
        let response = Frame::Simple("OK".to_string());
        dst.write_frame(&response).await ? ;
        Ok(())
    }
}

impl PrivCommand for Set {
    // this method is public under the same crate, not visible
    // to external 3rd-party crates
    fn parse_frames(parse: &mut Parse) -> AsyncResult<Box<dyn PubCommand>>
    {
        use ParseError::EndOfStream;
        // key / value fields are required
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;
        let mut _expire = None; // The expiration is optional
        match parse.next_string() { // Attempt to parse another string.
            Ok(s) if s.to_uppercase() == "EX" => {
                // An expiration is specified in seconds. The next value is an
                // integer.
                let secs = parse.next_int()?;
                _expire = Some(Duration::from_secs(secs));
            }
            Ok(s) if s.to_uppercase() == "PX" => {
                // An expiration is specified in milliseconds. The next value is
                // an integer.
                let ms = parse.next_int()?;
                _expire = Some(Duration::from_millis(ms));
            }
            // other set options are treated as format error
            Ok(_) => return Err("currently `SET` only supports the expiration option".into()),
            // The `EndOfStream` error indicates there is no further data to
            // parse. In this case, it is a normal run time situation and
            // indicates there are no specified `SET` options.
            Err(EndOfStream) => {}
            // All other errors are bubbled up, resulting in the connection
            // being terminated.
            Err(err) => return Err(err.into()),
        }
        let obj = Set{key, value, expire:_expire};
        Ok(Box::new(obj))
    }

    fn into_frame(self) -> Frame {
        let mut frm = Frame::array();
        frm.push_bulk(Bytes::from("set".as_bytes()));
        frm.push_bulk(Bytes::from(self.key.into_bytes()));
        frm.push_bulk(self.value);
        if let Some(ms) = self.expire {
            // Expirations in Redis procotol can be specified in two ways
            // 1. SET key value EX seconds
            // 2. SET key value PX milliseconds
            frm.push_bulk(Bytes::from("px".as_bytes()));
            frm.push_int(ms.as_millis() as u64);
        }
        frm
    }
} // end of Set class

