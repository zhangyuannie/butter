use zbus::dbus_proxy;

use crate::comm::BtrfsFilesystem;

#[dbus_proxy(
    interface = "org.zhangyuannie.Butter1",
    default_path = "/org/zhangyuannie/Butter1"
)]
trait Butter1 {
    fn list_filesystems(&self) -> zbus::Result<Vec<BtrfsFilesystem>>;
    fn enable_schedule(&self) -> zbus::Result<()>;
    fn disable_schedule(&self) -> zbus::Result<()>;
    fn schedule_state(&self) -> zbus::Result<String>;
}
