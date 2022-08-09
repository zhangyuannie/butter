use butter::daemon::interface::ButterdClient;
use std::{
    io::BufReader,
    process::{ChildStdin, ChildStdout},
    sync::{Arc, LockResult, Mutex, MutexGuard},
};

#[derive(Clone, Debug)]
pub struct Client(Arc<Mutex<ButterdClient>>);

impl Client {
    pub fn new(reader: BufReader<ChildStdout>, writer: ChildStdin) -> Self {
        Self(Arc::new(Mutex::new(ButterdClient { reader, writer })))
    }

    pub fn lock(&self) -> LockResult<MutexGuard<ButterdClient>> {
        self.0.lock()
    }
}
