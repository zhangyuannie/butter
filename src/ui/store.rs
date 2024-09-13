use std::collections::HashMap;

use anyhow::Context;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use uuid::Uuid;

use butterd::{
    FilesystemProxyBlocking, RuleProxyBlocking, ScheduleProxyBlocking, StorageProxyBlocking,
    ZPathBuf,
};
use zbus::{blocking::fdo::ObjectManagerProxy, proxy::ProxyDefault, zvariant::OwnedObjectPath};

use crate::object::{list::SubvolList, Filesystem, Rule, Subvolume};

mod imp {
    use std::cell::{OnceCell, RefCell};

    use butterd::FilesystemProxyBlocking;
    use gtk::{gio, glib, subclass::prelude::*};
    use zbus::blocking::Connection;

    use crate::object::{list::SubvolList, Filesystem, Rule};

    pub struct Store {
        pub conn: OnceCell<Connection>,
        pub model: SubvolList,
        pub filesystems: gio::ListStore,
        pub cur_fs: RefCell<Option<FilesystemProxyBlocking<'static>>>,
        pub rules: gio::ListStore,
    }

    impl Default for Store {
        fn default() -> Self {
            Self {
                conn: Default::default(),
                model: Default::default(),
                filesystems: gio::ListStore::new::<Filesystem>(),
                cur_fs: Default::default(),
                rules: gio::ListStore::new::<Rule>(),
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

    fn storage(&self) -> anyhow::Result<StorageProxyBlocking> {
        Ok(StorageProxyBlocking::new(&self.imp().conn.get().unwrap())?)
    }

    fn filesystem_manager(&self) -> anyhow::Result<ObjectManagerProxy> {
        Ok(ObjectManagerProxy::new(
            &self.imp().conn.get().unwrap(),
            StorageProxyBlocking::DESTINATION.unwrap(),
            StorageProxyBlocking::PATH.unwrap(),
        )?)
    }

    fn schedule(&self) -> anyhow::Result<ScheduleProxyBlocking> {
        Ok(ScheduleProxyBlocking::new(&self.imp().conn.get().unwrap())?)
    }

    fn rule_manager(&self) -> anyhow::Result<ObjectManagerProxy> {
        Ok(ObjectManagerProxy::new(
            &self.imp().conn.get().unwrap(),
            ScheduleProxyBlocking::DESTINATION.unwrap(),
            ScheduleProxyBlocking::PATH.unwrap(),
        )?)
    }

    pub fn refresh_subvolumes(&self) -> anyhow::Result<()> {
        let subvols = self
            .filesystem()
            .context("filesystem not selected")?
            .list_subvolumes()?;
        let subvols = {
            let mut map: HashMap<Uuid, Subvolume> = HashMap::with_capacity(subvols.len());
            for subvol in subvols {
                map.insert(*subvol.uuid.as_uuid(), Subvolume::new(subvol));
            }
            map
        };

        let model = self.model();
        model.clear();
        for (_, subvol) in subvols {
            model.insert(subvol);
        }

        Ok(())
    }

    pub fn refresh_filesystems(&self) {
        self.storage().unwrap().refresh().unwrap();
        let filesystems = self
            .filesystem_manager()
            .unwrap()
            .get_managed_objects()
            .unwrap();
        let mut filesystems: Vec<OwnedObjectPath> = filesystems.into_keys().collect();
        filesystems.sort_unstable_by(|a, b| a.cmp(b));

        let model = &self.imp().filesystems;
        model.remove_all();
        for path in filesystems {
            let proxy = FilesystemProxyBlocking::new(&self.imp().conn.get().unwrap(), path.clone())
                .unwrap();
            model.append(&Filesystem::new(
                path,
                proxy.uuid().unwrap().as_uuid().as_hyphenated().to_string(),
                proxy.label().unwrap(),
                proxy
                    .devices()
                    .unwrap()
                    .iter()
                    .map(|p| p.as_path().to_string_lossy().to_string())
                    .collect(),
            ));
        }
        if self.filesystem().is_none() {
            self.set_filesystem(
                model
                    .item(0)
                    .expect("No BTRFS found")
                    .downcast_ref()
                    .unwrap(),
            )
            .unwrap();
        }
    }

    pub fn filesystems(&self) -> &gio::ListStore {
        &self.imp().filesystems
    }

    pub fn filesystem(&self) -> Option<FilesystemProxyBlocking> {
        self.imp().cur_fs.borrow().clone()
    }

    pub fn set_filesystem(&self, fs: &Filesystem) -> anyhow::Result<()> {
        if let Some(cur_fs) = self.imp().cur_fs.borrow().as_ref() {
            if cur_fs.inner().path() == &fs.object_path().as_ref() {
                return Ok(());
            }
        }
        self.imp().cur_fs.replace(Some(FilesystemProxyBlocking::new(
            &self.imp().conn.get().unwrap(),
            fs.object_path().clone(),
        )?));
        self.refresh_subvolumes()?;

        Ok(())
    }

    pub fn delete_snapshots(&self, paths: Vec<ZPathBuf>) -> anyhow::Result<()> {
        self.storage()?.remove_subvolumes(paths)?;
        self.refresh_subvolumes()?;
        Ok(())
    }

    pub fn rename_snapshot(
        &self,
        before_path: ZPathBuf,
        after_path: ZPathBuf,
    ) -> anyhow::Result<()> {
        self.storage()?.move_subvolume(before_path, after_path)?;
        self.refresh_subvolumes()?;
        Ok(())
    }

    pub fn create_snapshot(
        &mut self,
        src: ZPathBuf,
        dest: ZPathBuf,
        readonly: bool,
    ) -> anyhow::Result<()> {
        self.storage()?.create_snapshot(src, dest, readonly)?;
        self.refresh_subvolumes()?;
        Ok(())
    }

    pub fn is_schedule_enabled(&self) -> bool {
        self.schedule().unwrap().is_enabled().unwrap()
    }

    pub fn set_is_schedule_enabled(&self, is_enabled: bool) -> anyhow::Result<()> {
        Ok(self.schedule()?.set_is_enabled(is_enabled)?)
    }

    pub fn rule_model(&self) -> &gio::ListStore {
        &self.imp().rules
    }

    pub fn refresh_rules(&self) -> anyhow::Result<()> {
        self.schedule()?.refresh()?;
        let rules = self.rule_manager()?.get_managed_objects()?;
        let mut rules: Vec<OwnedObjectPath> = rules.into_keys().collect();
        rules.sort_unstable_by(|a, b| a.cmp(b));
        let model = self.rule_model();
        model.remove_all();
        for path in rules {
            let proxy = RuleProxyBlocking::new(&self.imp().conn.get().unwrap(), path.clone())?;
            let name = proxy.name()?;
            let is_enabled = proxy.is_enabled()?;
            let config = proxy.config()?;
            model.append(&Rule::new(path, name, is_enabled, config));
        }
        Ok(())
    }

    pub fn delete_rule(&self, rule: &Rule) -> anyhow::Result<()> {
        self.schedule()?.remove_rule(rule.name())?;
        self.refresh_rules()?;
        Ok(())
    }

    pub fn create_rule(&self, rule: &Rule) -> anyhow::Result<()> {
        self.schedule()?
            .create_rule(rule.name(), rule.config().clone())?;
        self.refresh_rules()?;
        Ok(())
    }

    pub fn update_rule(&self, prev: &Rule, next: &Rule) -> anyhow::Result<()> {
        let path = if prev.name() != next.name() {
            self.schedule()?.move_rule(prev.name(), next.name())?
        } else {
            prev.object_path().clone()
        };

        let proxy = RuleProxyBlocking::new(&self.imp().conn.get().unwrap(), path)?;
        if *prev.config() != *next.config() {
            proxy.set_config(next.config().clone())?;
        }
        self.refresh_rules()?;
        Ok(())
    }
}
