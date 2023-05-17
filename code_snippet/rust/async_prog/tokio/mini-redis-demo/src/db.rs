use std::collections::{HashMap, hash_map};
use std::sync::{Arc, Mutex};
use std::io::{Result as IoResult, Error as IoError, ErrorKind};
use bytes::Bytes;
use tokio::sync::broadcast;

struct InnerDataStore {
    keyval: HashMap<String, Vec<u8>>,
    pubsub: HashMap<String, broadcast::Sender<Bytes>>,
}

pub struct FakeDatabase {
    shared : Arc<Mutex<InnerDataStore>>,
}

impl Clone for FakeDatabase {
    fn clone(&self) -> Self {
        let shr_state = Arc::clone(&self.shared);
        Self{ shared: shr_state }
    }
}
impl Drop for FakeDatabase {
    fn drop(&mut self) {
        let num_refs = Arc::strong_count(&self.shared);
        if num_refs == 1 {
            let mut fdb = self.shared.lock().unwrap();
            fdb.keyval.clear();
            fdb.keyval.shrink_to_fit();
        }
        println!("[db][drop] shared, ref count:{}, {}",
            num_refs,  Arc::weak_count(&self.shared) );
    }
}

impl FakeDatabase {
    pub fn new() -> Self {
        let _inner_store = InnerDataStore{ keyval:HashMap::new(),
            pubsub:HashMap::new() };
        let shr_state = Arc::new(Mutex::new(_inner_store));
        Self{ shared: shr_state }
    }
    pub fn set(&self, k:&str, v:Vec<u8>) -> IoResult<()>
    {
        if let Ok(mut fdb) = self.shared.lock() {
            // the hashmap object also needs to be owner of the
            // key / value stored in frame without moving them.
            let key = k.to_string();
            let value = v;
            fdb.keyval.insert(key, value); // may return previously inserted value
            Ok(())
        } else {
            let e = IoError::new( ErrorKind::ResourceBusy,
                                  "failed to acquire db lock");
            Err(e)
        }
    }
    pub fn get(&self, k:&str) -> IoResult<Option<Vec<u8>>>
    {
        if let Ok(fdb) = self.shared.lock() {
            if let Some(v) = fdb.keyval.get(k) {
                Ok(Some(v.clone()))
            } else {
                Ok(None)
            }
        } else {
            let e = IoError::new( ErrorKind::ResourceBusy,
                                  "failed to acquire db lock");
            Err(e)
        }
    }
    pub(crate) fn publish(&self, chn:&str, msg:&Bytes) -> IoResult<usize> {
        if let Ok(fdb) = self.shared.lock() {
            if let Some(sender) = fdb.pubsub.get(chn) {
                let msg = msg.clone();
                let num_subs = sender.send(msg).unwrap_or(0);
                Ok(num_subs)
            } else {
                let e = IoError::new( ErrorKind::NotFound,
                                      "channel not exists");
                Err(e)
            }
        } else {
            let e = IoError::new( ErrorKind::ResourceBusy,
                                  "failed to acquire db lock");
            Err(e)
        }
    }
    pub(crate) fn subscribe(&self, chn:String) -> IoResult<broadcast::Receiver<Bytes>>
    {
        if let Ok(mut fdb) = self.shared.lock() {
            let recver = match fdb.pubsub.entry(chn) {
                hash_map::Entry::Occupied(e) => e.get().subscribe(),
                hash_map::Entry::Vacant(e) => {
                    let (_sender, _recver) = broadcast::channel(30);
                    e.insert(_sender);
                    _recver
                },
            };
            Ok(recver)
        } else {
            let e = IoError::new( ErrorKind::ResourceBusy,
                                  "failed to acquire db lock");
            Err(e)
        }
    }
} // end of FakeDatabase

