#![feature(io_error_more)]

mod connection;
pub use connection::Connection;

mod parse; // not public module / types
use parse::{Parse, ParseError};

pub mod frame;
pub use frame::{Frame}; 

pub mod clients;
pub use clients::{Client}; 

pub mod db;
pub mod cmd;

pub const DEFAULT_PORT:u16 = 6379;

// the AsyncError could be used as return type of any async function.
// why adding `Send` trait ?
// - the ownership of the error should be able to transferred between threads
// why adding `Sync` trait ?
// - the error can be accessed between threads
pub type AsyncError = Box<dyn std::error::Error + Send + Sync>;

// specialized result type
pub type AsyncResult<T> = Result<T, AsyncError>;

