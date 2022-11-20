use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::{dbus_interface, zvariant::Type, DBusError};
use zbus_polkit::policykit1;

use crate::btrfs;

#[derive(DBusError, Debug)]
#[dbus_error(prefix = "org.zhangyuannie.Butter1")]
enum Error {
    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),
    Failed(String),
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Failed(e.to_string())
    }
}
pub struct Service<'c> {
    pub polkit: policykit1::AuthorityProxy<'c>,
}

#[dbus_interface(name = "org.zhangyuannie.Butter1")]
impl Service<'static> {
    fn list_filesystems(&self) -> Result<Vec<BtrfsFilesystem>, Error> {
        let ret = btrfs::read_all_mounted_btrfs_fs().context("failed to read mounted btrfs fs")?;
        Ok(ret)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Type)]
pub struct BtrfsFilesystem {
    pub label: String,
    pub uuid: Uuid,
    // TODO: PathBuf
    pub devices: Vec<String>,
}
