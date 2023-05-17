use std::io::Cursor;
use bytes::{BytesMut, Buf};

// A single call to `TcpStream.read()` or `TcpStream.write()` will fetch
// or deliver arbitrary amount of data from/to low-level socket respectively.
use tokio::net::TcpStream;

// `AsyncReadExt` is required when reading raw data from internal buffer
//  by calling `BufWriter.read_buf(&BytesMut)`.
//  `BufWriter` implements both the traits `AsyncReadExt` and `AsyncRead`,
//  since `read_buf(...)` is declared only in `AsyncReadExt`, the trait
//  is imported as telling `BufWriter` to apply the methods on `AsyncReadExt`
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};

use crate::{AsyncResult};
use crate::frame::{self, Frame};

pub struct Connection {
    // `TcpStream` decorated with `BufWriter`, it is possible to perform write
    // operations directly on `TcpStream`, but it could be better to write the
    // content to some buffer then flush it to `TcpStream` as soon as it is full
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new (socket:TcpStream) -> Self {
        let buf_nbytes:usize = 1usize << 10;
        Self{
            stream:BufWriter::new(socket),
            buffer:BytesMut::with_capacity(buf_nbytes)
        }
    }

    fn parse_frame(&mut self) -> AsyncResult<Option<Frame>>
    {
        let sliced:&[u8] = &self.buffer[..];
        // use the cursor type provided by standard library, no need to
        // implement your own
        let mut _cursor = Cursor::new(sliced);
        // the method below check format of the read raw data, whether
        // it can be converted to valid mini-redis frame.
        match Frame::check(& mut _cursor) { 
            Ok(_) => {
                // `check()` would also modify the given buffer array and
                // internal pointer of `Cursor` object.
                let len = _cursor.position() as usize;
                _cursor.set_position(0);
                // if error happens in the middle of parsing, immediately
                // return with the error, otherwise automatically unwrap
                // the returned result.
                let frm:Frame = Frame::parse(& mut _cursor)?;
                // Discard the data regardless of whether it is parsed
                // successfully or not. Note `advance()` requires `bytes::Buf`
                // to be imported
                self.buffer.advance(len);
                Ok(Some(frm))
            },
            Err(frame::Error::Incomplete) => { Ok(None) },
            // you can also convert the type by calling `e.into()`
            // `Error` type doesn't implement `into()` method, what is the magic
            // behind that ? (TODO)
            Err(e) => { Err(Box::new(e)) },
        }
    } // end of parse_frame

    pub async fn read_frame(&mut self) -> AsyncResult<Option<Frame>>
    {
        // try parsing the frame from internal read buffer, it is possible that the
        // buffer currently contains a partial frame, or multiple small-sized frames
        loop { // question mark at the end automatically unwraps returned result
            if let Some(frm) = self.parse_frame() ? {
                break Ok(Some(frm))
            } // return one parsed frame at once
            // Try loading more bytes in TCP stream, copy it to the given buffer.
            // Note the API doc of Tokio crate is confusing, for `read_buf()`,
            // the return type should be `tokio::io::Result<usize>`,
            // and it should be asynchronous function.
            let nread = self.stream.read_buf(& mut self.buffer).await ?;
            if 0 == nread { // `zero` indicates end of stream
                if self.buffer.is_empty() {
                    break Ok(None)
                } else {
                    break Err("connection reset by peer".into())
                }
            } // if new bytes were loaded, they'd be parsed in next iteration.
        } // return value as soon as break from loop
    } // end of read_frame

    // `Result` type in stardard I/O module already implements traits `Send`
    // and `Sync`, therefore no need to use `AsyncResult<T>`
    pub async fn write_frame(&mut self, frm:&Frame) -> std::io::Result<()>
    {
        match frm {
            Frame::Array(frmlist) => {
                // craft 2 bytes ahead, tell the peer it is array of frames
                // at low-level message
                self.stream.write_u8(b'*').await?;
                self.write_decimal(frmlist.len() as u64).await?;
                for f in frmlist {
                    self.write_single_frame(f).await?;
                } // should not take ownership at here ?
            },
            _others => self.write_single_frame(&frm).await? ,
        };
        // flush the content in BufWriter to the TCP stream
        self.stream.flush().await
    }

    async fn write_decimal(&mut self, val: u64) -> std::io::Result<()>
    {
        use std::io::Write;
        // Convert the value to a string
        let mut buf = [0u8; 20];
        let mut _cursor = Cursor::new(&mut buf[..]);
        write!(&mut _cursor, "{}", val)?;
        let pos = _cursor.position() as usize;
        self.stream.write_all(&_cursor.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;
        Ok(())
    }

    async fn write_single_frame(&mut self, frm:&Frame) -> std::io::Result<()>
    {
        match frm {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            },
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            },
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val as u64).await?;
            },
            Frame::Bulk(val) => {
                let _val:&[u8] = &val;
                let sz = val.len();
                self.stream.write_u8(b'$').await?;
                self.write_decimal(sz as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            },
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            },
            // nested array not supported
            Frame::Array(_) => unreachable!(),
        }
        Ok(())
    }
} // end of Connection implementation

