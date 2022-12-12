use std::path::Path;

use butterd::{BtrfsFilesystem, Subvolume};
use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.zhangyuannie.Butter1",
    default_path = "/org/zhangyuannie/Butter1"
)]
trait Butter1 {
    fn list_filesystems(&self) -> zbus::Result<Vec<BtrfsFilesystem>>;
    fn list_subvolumes(&self, fs: &BtrfsFilesystem) -> zbus::Result<Vec<Subvolume>>;
    fn enable_schedule(&self) -> zbus::Result<()>;
    fn disable_schedule(&self) -> zbus::Result<()>;
    fn schedule_state(&self) -> zbus::Result<String>;
    fn move_subvolume(&self, src_mnt: &Path, dst_mnt: &Path) -> zbus::Result<()>;
    fn delete_subvolume(&self, mnt: &Path) -> zbus::Result<()>;
    fn create_snapshot(self, src_mnt: &Path, dst_mnt: &Path, flags: i32)
        -> zbus::Result<Subvolume>;
}
