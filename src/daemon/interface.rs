use std::{
    fmt::{Display, Write},
    path::PathBuf,
    result,
    time::SystemTime,
};

use libc::c_int;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

pub trait DaemonInterface {
    fn list_filesystems(&mut self) -> Result<Vec<BtrfsFilesystem>>;
    fn filesystem(&mut self) -> Option<Uuid>;
    fn set_filesystem(&mut self, fs: BtrfsFilesystem) -> Result<bool>;
    fn list_subvolumes(&mut self) -> Result<Vec<Subvolume>>;
    fn move_subvolume(&mut self, from: PathBuf, to: PathBuf) -> Result<()>;
    fn delete_subvolume(&mut self, path: PathBuf) -> Result<()>;
    fn create_snapshot(
        &mut self,
        src: PathBuf,
        dest: PathBuf,
        flags: libbtrfsutil::CreateSnapshotFlags,
    ) -> Result<Subvolume>;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    ListFilesystems,
    Filesystems,
    SetFilesystem(BtrfsFilesystem),
    ListSubvolumes,
    MoveSubvolume(PathBuf, PathBuf),
    DeleteSubvolume(PathBuf),
    CreateSnapshot(PathBuf, PathBuf, c_int),
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct BtrfsFilesystem {
    pub label: Option<String>,
    pub uuid: Uuid,
    pub devices: Vec<PathBuf>,
}
