use std::{
    fmt::{Display, Write},
    path::PathBuf,
    result,
    time::SystemTime,
};

use butterd::BtrfsFilesystem;
use libc::c_int;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{json_file::JsonFile, schedule::Schedule};

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    msg: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        let mut msg = String::new();
        write!(&mut msg, "{:#}", err).expect("failed to convert an error");
        Error { msg }
    }
}

impl Error {
    pub fn new<T: Into<String>>(msg: T) -> Error {
        Error { msg: msg.into() }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = result::Result<T, Error>;

#[rpc::service]
pub trait Butterd {
    fn schedules() -> Result<Vec<JsonFile<Schedule>>>;
    fn fs_rename(from: PathBuf, to: PathBuf) -> Result<()>;
    fn flush_schedule(schedule: JsonFile<Schedule>) -> Result<()>;
    fn fs_remove_file(path: PathBuf) -> Result<()>;
}
