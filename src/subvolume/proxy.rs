use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::{dbus_proxy, zvariant::Type};

#[dbus_proxy(
    interface = "org.zhangyuannie.Butter1",
    default_path = "/org/zhangyuannie/Butter1"
)]
trait Butter1 {
    fn list_filesystems(&self) -> zbus::Result<Vec<BtrfsFilesystem>>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Type)]
pub struct BtrfsFilesystem {
    pub label: String,
    pub uuid: Uuid,
    // TODO: PathBuf
    pub devices: Vec<String>,
}
