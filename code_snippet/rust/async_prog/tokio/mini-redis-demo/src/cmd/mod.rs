use crate::{Frame, Connection, Parse, AsyncResult};
use crate::db::FakeDatabase;
use crate::cmd::private_part::Command as PrivCommand;

use async_trait::async_trait;

mod get;
pub use get::Get;

mod set;
pub use set::Set;

mod publish;
pub use publish::Publish;

mod subscribe;
pub use subscribe::{ Subscribe, Unsubscribe, 
    CommonInit as SubscribeCommonInit};

mod unknown;
pub use unknown::Unknown;

// this function below cannot be a trait method. Rust compiler checks trait
// method using a set of object-safety rules :
// - `self` syntax has to indicate any instance whose the size is known,
//    it is essential to append `where Self: Sized` to the function signature
//    if there is no `self` syntax in the signature
// - requires caller to annotate concrete type when calling the method.
//   (impossible if size is unknown at compile time)
pub fn from_frame(frm:Frame) -> AsyncResult<Box<dyn Command>>
{ // `Parse` provides a "cursor" like API which makes parsing
  // the command easier
    let mut parsed = Parse::new(frm)?;
    let command_name = parsed.next_string()?.to_lowercase();
    let cmd_name_ref = &command_name[..];
    let obj = match cmd_name_ref {
        "get" => Get::parse_frames(&mut parsed)?,
        "set" => Set::parse_frames(&mut parsed)?,
        "publish" => Publish::parse_frames(&mut parsed)?,
        "subscribe" => Subscribe::parse_frames(&mut parsed)?,
        _others => Unknown::parse_frames(&mut parsed)?,
    };
    // Check if there is any remaining unconsumed fields in the `Parse`
    // value. If fields remain, this indicates an unexpected frame format
    // and an error is returned.
    parsed.finish()?;
    // The command has been successfully parsed
    Ok(obj)
}

// It is unnecessary to add visibility qualifier like `pub` or `pub crate`
// in concrete type methods implementing any trait which defines public abstract
// functions, the visibility of these concrete types will be implied by
// the trait type.

// trait `Send` is required for saving the instance of concrete type
// when caller of async function is `awaiting`
#[async_trait]
pub trait Command : PrivCommand + Send
{
    // 1. visibility of the trait implies to all its methods
    // 2. caller cannot move the ownership of the instance through all trait methods
    //    because the caller doesn't know the size.
    async fn apply(&self, db:&FakeDatabase, dst:&mut Connection) -> AsyncResult<()>;
} // end of trait


pub(crate) mod private_part {
    use std::io::{Error as IoError, ErrorKind};
    use crate::{Frame, Parse, AsyncResult};
    use crate::cmd::Command as PubCommand;
    
    pub trait Command {
        fn parse_frames(_parse: &mut Parse) -> AsyncResult<Box<dyn PubCommand>>
            where Self: Sized // required for object safety
        {
            let e = IoError::new( ErrorKind::Unsupported, "");
            Err(Box::new(e))
        }
        fn into_frame(self) -> Frame;
    }
} // end of trait

