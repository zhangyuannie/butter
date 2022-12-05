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
    fn filesystem() -> Option<Uuid>;
    fn set_filesystem(fs: BtrfsFilesystem) -> Result<bool>;
    fn list_subvolumes() -> Result<Vec<Subvolume>>;
    fn move_subvolume(from: PathBuf, to: PathBuf) -> Result<()>;
    fn delete_subvolume(path: PathBuf) -> Result<()>;
    fn create_snapshot(src: PathBuf, dest: PathBuf, flags: c_int) -> Result<Subvolume>;
    fn schedules() -> Result<Vec<JsonFile<Schedule>>>;
    fn fs_rename(from: PathBuf, to: PathBuf) -> Result<()>;
    fn flush_schedule(schedule: JsonFile<Schedule>) -> Result<()>;
    fn fs_remove_file(path: PathBuf) -> Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subvolume {
    pub path: PathBuf,
    pub uuid: Uuid,
    pub created: SystemTime,
    pub snapshot_source_uuid: Option<Uuid>,
}

impl Default for Subvolume {
    fn default() -> Self {
        Self {
            path: Default::default(),
            uuid: Default::default(),
            created: SystemTime::UNIX_EPOCH,
            snapshot_source_uuid: Default::default(),
        }
    }
}
