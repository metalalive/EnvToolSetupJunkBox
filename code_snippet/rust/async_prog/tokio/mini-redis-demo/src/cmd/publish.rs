use bytes::Bytes;
use async_trait::async_trait;

use crate::{Connection, AsyncResult, Parse, Frame};
use crate::cmd::{Command as PubCommand, private_part::Command as PrivCommand};
use crate::db::FakeDatabase;

#[derive(Debug)]
pub struct Publish {
    channel: String,
    message: Bytes,
}

impl Publish {
    pub(crate) fn new(chn:impl ToString, msg:Bytes) -> Self {
        Self{channel: chn.to_string(), message:msg}
    }
}

#[async_trait]
impl PubCommand for Publish {
    async fn apply(&self, db:&FakeDatabase, dst:&mut Connection) -> AsyncResult<()>
    {
        let response = match db.publish(&self.channel, &self.message)
        {
            Ok(num_subsribers) => Frame::Integer(num_subsribers as u64),
            Err(e) => Frame::Error(e.to_string()),
        };
        dst.write_frame(&response).await ?;
        Ok(())
    }
} // end of impl PubCommand

impl PrivCommand for Publish {
    fn parse_frames(parse: &mut Parse) -> AsyncResult<Box<dyn PubCommand>>
    {
        let channel = parse.next_string()?;
        let message = parse.next_bytes()?;
        let out = Box::new(Self{channel, message});
        Ok(out)
    }
    fn into_frame(self) -> Frame
    {
        let mut out = Frame::array();
        out.push_bulk(Bytes::from("publish".to_string()));
        out.push_bulk(Bytes::from(self.channel.into_bytes()));
        out.push_bulk(self.message);
        out
    }
} // end of impl PrivCommand
