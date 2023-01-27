use std::path::{Path, PathBuf};

use zbus::dbus_proxy;

use crate::{filesystem::Filesystem, rule::Rule, subvolume::Subvolume};

#[dbus_proxy(
    interface = "org.zhangyuannie.Butter1",
    default_path = "/org/zhangyuannie/Butter1"
)]
trait Butter1 {
    fn list_filesystems(&self) -> zbus::Result<Vec<Filesystem>>;
    fn list_subvolumes(&self, fs: &Filesystem) -> zbus::Result<Vec<Subvolume>>;
    fn enable_schedule(&self) -> zbus::Result<()>;
    fn disable_schedule(&self) -> zbus::Result<()>;
    fn schedule_state(&self) -> zbus::Result<String>;
    fn move_subvolume(&self, src_mnt: &Path, dst_mnt: &Path) -> zbus::Result<()>;
    fn delete_subvolumes(&self, mnts: &[PathBuf]) -> zbus::Result<()>;
    fn create_snapshot(
        self,
        src_mnt: &Path,
        dst_mnt: &Path,
        readonly: bool,
    ) -> zbus::Result<Subvolume>;
    fn list_rules(&self) -> zbus::Result<Vec<Rule>>;
    fn update_rule(&self, prev: &Rule, next: &Rule) -> zbus::Result<()>;
    fn delete_rule(&self, rule: &Rule) -> zbus::Result<()>;
    fn create_rule(&self, rule: &Rule) -> zbus::Result<()>;
}
