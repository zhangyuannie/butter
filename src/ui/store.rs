use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gtk::{gio, glib, subclass::prelude::*};
use uuid::Uuid;

use crate::daemon::proxy::Butter1ProxyBlocking;
use crate::filesystem::{Filesystem, GFilesystem};
use crate::rule::{GRule, Rule};
use crate::subvolume::{GSubvolume, SubvolList};

mod imp {
    use std::cell::RefCell;

    use gtk::{gio, glib, prelude::*, subclass::prelude::*};
    use once_cell::sync::OnceCell;
    use zbus::blocking::Connection;

    use crate::{filesystem::GFilesystem, rule::GRule, subvolume::SubvolList};

    pub struct Store {
        pub conn: OnceCell<Connection>,
        pub model: SubvolList,
        pub filesystems: gio::ListStore,
        pub cur_fs: RefCell<Option<GFilesystem>>,
        pub rules: gio::ListStore,
    }

    impl Default for Store {
        fn default() -> Self {
            Self {
                conn: Default::default(),
                model: Default::default(),
                filesystems: gio::ListStore::new::<GFilesystem>(),
                cur_fs: Default::default(),
                rules: gio::ListStore::new::<GRule>(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Store {
        const NAME: &'static str = "BtrStore";
        type Type = super::Store;
    }

    impl ObjectImpl for Store {}
}

glib::wrapper! {
    pub struct Store(ObjectSubclass<imp::Store>);
}

impl Store {
    pub fn new() -> anyhow::Result<Self> {
        let ret: Self = glib::Object::new();
        ret.imp()
            .conn
            .set(zbus::blocking::Connection::system()?)
            .unwrap();
        ret.refresh_filesystems();
        ret.refresh_subvolumes()?;
        ret.refresh_rules()?;
        Ok(ret)
    }

    pub fn model(&self) -> SubvolList {
        self.imp().model.clone()
    }

    fn butterd(&self) -> anyhow::Result<Butter1ProxyBlocking> {
        let butterd = Butter1ProxyBlocking::new(&self.imp().conn.get().unwrap())?;
        Ok(butterd)
    }

    pub fn refresh_subvolumes(&self) -> anyhow::Result<()> {
        if let Some(cur_fs) = self.imp().cur_fs.borrow().as_ref() {
            let subvols = self.butterd()?.list_subvolumes(cur_fs.data())?;
            let subvols = {
                let mut map: HashMap<Uuid, GSubvolume> = HashMap::with_capacity(subvols.len());
                for subvol in subvols {
                    map.insert(subvol.uuid, GSubvolume::new(subvol));
                }
                map
            };

            // populate parent now
            for (_, subvol) in &subvols {
                if let Some(uuid) = subvol.parent_uuid() {
                    subvol.set_parent(subvols.get(&uuid));
                }
            }

            let model = self.model();
            model.clear();
            for (_, subvol) in subvols {
                model.insert(subvol);
            }
        }
        Ok(())
    }

    pub fn refresh_filesystems(&self) {
        let filesystems = self.butterd().unwrap().list_filesystems().unwrap();
        if self.filesystem().is_none() {
            self.set_filesystem(filesystems[0].clone()).unwrap();
        }

        let model = &self.imp().filesystems;
        model.remove_all();
        for fs in filesystems {
            model.append(&GFilesystem::new(fs));
        }
    }

    pub fn filesystems(&self) -> &gio::ListStore {
        &self.imp().filesystems
    }

    pub fn filesystem(&self) -> Option<Uuid> {
        self.imp()
            .cur_fs
            .borrow()
            .as_ref()
            .and_then(|x| Some(x.uuid()))
    }

    pub fn set_filesystem(&self, fs: Filesystem) -> anyhow::Result<()> {
        if let Some(cur_fs) = self.imp().cur_fs.borrow().as_ref() {
            if cur_fs.uuid() == fs.uuid {
                return Ok(());
            }
        }
        self.imp().cur_fs.replace(Some(GFilesystem::new(fs)));
        self.refresh_subvolumes()?;

        Ok(())
    }

    pub fn delete_snapshots(&self, paths: &[PathBuf]) -> anyhow::Result<()> {
        self.butterd()?.delete_subvolumes(paths)?;
        self.refresh_subvolumes()?;
        Ok(())
    }

    pub fn rename_snapshot(&self, before_path: &Path, after_path: &Path) -> anyhow::Result<()> {
        self.butterd()?.move_subvolume(before_path, after_path)?;
        self.refresh_subvolumes()?;
        Ok(())
    }

    pub fn create_snapshot(
        &mut self,
        src: &Path,
        dest: &Path,
        readonly: bool,
    ) -> anyhow::Result<()> {
        self.butterd()?.create_snapshot(&src, &dest, readonly)?;
        self.refresh_subvolumes()?;
        Ok(())
    }

    pub fn is_schedule_enabled(&self) -> bool {
        self.butterd().unwrap().schedule_state().unwrap() == "active"
    }

    pub fn set_is_schedule_enabled(&self, is_enabled: bool) -> anyhow::Result<()> {
        if is_enabled {
            Ok(self.butterd()?.enable_schedule()?)
        } else {
            Ok(self.butterd()?.disable_schedule()?)
        }
    }

    pub fn rule_model(&self) -> &gio::ListStore {
        &self.imp().rules
    }

    pub fn refresh_rules(&self) -> anyhow::Result<()> {
        let mut rules = self.butterd()?.list_rules()?;
        rules.sort_by(|a, b| a.path.cmp(&b.path));
        let model = self.rule_model();
        model.remove_all();
        for rule in rules {
            model.append(&GRule::new(rule));
        }
        Ok(())
    }

    pub fn delete_rule(&self, rule: &Rule) -> anyhow::Result<()> {
        self.butterd()?.delete_rule(rule)?;
        self.refresh_rules()?;
        Ok(())
    }

    pub fn create_rule(&self, rule: &Rule) -> anyhow::Result<()> {
        self.butterd()?.create_rule(rule)?;
        self.refresh_rules()?;
        Ok(())
    }

    pub fn update_rule(&self, prev: &Rule, next: &Rule) -> anyhow::Result<()> {
        self.butterd()?.update_rule(prev, next)?;
        self.refresh_rules()?;
        Ok(())
    }
}
