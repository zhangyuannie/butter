use crate::schedule_repo::ScheduleRepo;
use crate::subvolume::{GSubvolume, SubvolList};

use butterd::BtrfsFilesystem;
use glib::once_cell::sync::OnceCell;
use gtk::prelude::*;
use gtk::{gio, glib, subclass::prelude::*};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{ChildStdin, ChildStdout};
use uuid::Uuid;

use crate::{client::Client, subvolume::g_btrfs_filesystem::GBtrfsFilesystem};

use super::proxy::Butter1ProxyBlocking;

mod imp {
    use super::*;

    use std::cell::RefCell;

    pub struct SubvolumeManager {
        pub daemon: OnceCell<Client>,
        pub model: SubvolList,
        pub filesystems: gio::ListStore,
        pub cur_fs: RefCell<Option<GBtrfsFilesystem>>,
    }

    impl Default for SubvolumeManager {
        fn default() -> Self {
            Self {
                daemon: Default::default(),
                model: Default::default(),
                filesystems: gio::ListStore::new(GBtrfsFilesystem::static_type()),
                cur_fs: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubvolumeManager {
        const NAME: &'static str = "SubvolumeManager";
        type Type = super::SubvolumeManager;
    }

    impl ObjectImpl for SubvolumeManager {}
}

glib::wrapper! {
    pub struct SubvolumeManager(ObjectSubclass<imp::SubvolumeManager>);
}

impl SubvolumeManager {
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        let ret: Self = glib::Object::new(&[]);
        let imp = ret.imp();

        imp.daemon
            .set(Client::new(BufReader::new(stdout), stdin))
            .expect("Failed to set daemon");

        ret.refresh();
        ret
    }

    pub fn model(&self) -> &SubvolList {
        &self.imp().model
    }

    pub fn refresh(&self) {
        self.refresh_filesystems();
        self.refresh_subvolumes();
    }

    fn butterd(&self) -> anyhow::Result<Butter1ProxyBlocking> {
        let conn = zbus::blocking::Connection::system()?;
        let butterd = Butter1ProxyBlocking::new(&conn)?;
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
        let conn = zbus::blocking::Connection::system().unwrap();
        let butterd = Butter1ProxyBlocking::new(&conn).unwrap();
        let filesystems = butterd.list_filesystems().unwrap();
        if self.filesystem().is_none() {
            self.set_filesystem(filesystems[0].clone()).unwrap();
        }

        let model = &self.imp().filesystems;
        model.remove_all();
        for fs in filesystems {
            model.append(&GBtrfsFilesystem::new(fs));
        }
    }

    pub fn filesystems(&self) -> &gio::ListStore {
        &self.imp().filesystems
    }

    pub fn filesystem(&self) -> Option<Uuid> {
        self.imp().cur_fs.borrow().as_ref().and_then(|x| Some(x.uuid()))
    }

    pub fn set_filesystem(&self, fs: BtrfsFilesystem) -> anyhow::Result<()> {
        if let Some(cur_fs) = self.imp().cur_fs.borrow().as_ref() {
            if cur_fs.uuid() == fs.uuid {
                return Ok(());
            }
        }
        self.imp().cur_fs.replace(Some(GBtrfsFilesystem::new(fs)));
        self.refresh_subvolumes();

        Ok(())
    }

    pub fn delete_snapshot(&self, path: &Path) -> anyhow::Result<()> {
        self.butterd()?.delete_subvolume(path)?;
        self.refresh_subvolumes();
        Ok(())
    }

    pub fn rename_snapshot(&self, before_path: &Path, after_path: &Path) -> anyhow::Result<()> {
        self.butterd()?.move_subvolume(before_path, after_path)?;
        self.refresh_subvolumes();
        Ok(())
    }

    pub fn create_snapshot(
        &mut self,
        src: &Path,
        dest: &Path,
        readonly: bool,
    ) -> anyhow::Result<()> {
        let flags = if readonly {
            libbtrfsutil::CreateSnapshotFlags::READ_ONLY.bits()
        } else {
            0
        };
        self.butterd()?.create_snapshot(&src, &dest, flags)?;
        self.refresh_subvolumes();
        Ok(())
    }

    pub fn is_schedule_enabled(&self) -> bool {
        let conn = zbus::blocking::Connection::system().unwrap();
        let butterd = Butter1ProxyBlocking::new(&conn).unwrap();
        butterd.schedule_state().unwrap() == "active"
    }

    pub fn set_is_schedule_enabled(&self, is_enabled: bool) -> anyhow::Result<()> {
        let conn = zbus::blocking::Connection::system()?;
        let butterd = Butter1ProxyBlocking::new(&conn)?;
        if is_enabled {
            Ok(butterd.enable_schedule()?)
        } else {
            Ok(butterd.disable_schedule()?)
        }
    }

    pub fn schedule_repo(&self) -> ScheduleRepo {
        ScheduleRepo::new(self.imp().daemon.get().unwrap().clone())
    }
}
