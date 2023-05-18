use crate::{Connection, AsyncResult, Frame, Parse, SingleRequestShutdown};
use crate::db::FakeDatabase;
use crate::cmd::{Command as PubCommand, private_part::Command as PrivCommand};

use async_trait::async_trait;

#[derive(Debug)]
pub struct Unknown {
    name: String,
}

impl Unknown {
    pub fn new(k: impl ToString) -> Self {
        Self {name: k.to_string(),}
    }
}

impl PrivCommand for Unknown {
    fn parse_frames(_parse: &mut Parse) -> AsyncResult<Box<dyn PubCommand>>
    {
        let obj = Self{name:"unknown".to_string()};
        Ok(Box::new(obj)) 
    }

    fn into_frame(self) -> Frame
    { Frame::Null }
}
    
#[async_trait]
impl PubCommand for Unknown {
    async fn apply(&self, _fdb: &FakeDatabase, dst: &mut Connection,
                   _ :&mut SingleRequestShutdown) -> AsyncResult<()>
    {
        let detail = format!("frame not supported, type:{}", self.name);
        let response = Frame::Error(detail);
        dst.write_frame(&response).await ? ;
        Ok(())
    }
}
