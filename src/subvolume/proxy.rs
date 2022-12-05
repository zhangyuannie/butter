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
}
