use crate::subvolume::{GSubvolume, SubvolList};

use butter::json_file::JsonFile;
use butter::schedule::Schedule;
use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;

use butter::daemon::interface::DaemonInterfaceClient;
use glib::once_cell::sync::OnceCell;
use gtk::{gio, glib, subclass::prelude::*};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{ChildStdin, ChildStdout};
use std::sync::Mutex;
use uuid::Uuid;

use butter::daemon::interface::BtrfsFilesystem;

use super::g_btrfs_filesystem::GBtrfsFilesystem;

mod imp {
    use crate::subvolume::g_btrfs_filesystem::GBtrfsFilesystem;

    use super::*;

    pub struct SubvolumeManager {
        pub daemon: OnceCell<Mutex<DaemonInterfaceClient>>,
        pub model: SubvolList,
        pub filesystems: gio::ListStore,
    }

    impl Default for SubvolumeManager {
        fn default() -> Self {
            Self {
                daemon: Default::default(),
                model: Default::default(),
                filesystems: gio::ListStore::new(GBtrfsFilesystem::static_type()),
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
        let ret: Self = glib::Object::new(&[]).expect("Failed to create SubvolumeManager");
        let imp = ret.imp();

        imp.daemon
            .set(Mutex::new(DaemonInterfaceClient {
                reader: BufReader::new(stdout),
                writer: stdin,
            }))
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

    pub fn refresh_subvolumes(&self) {
        let daemon = self.imp().daemon.get().unwrap();
        let subvols = daemon.lock().unwrap().list_subvolumes().unwrap();
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

    pub fn refresh_filesystems(&self) {
        let daemon = self.imp().daemon.get().unwrap();
        let filesystems = daemon.lock().unwrap().list_filesystems().unwrap();
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
        let daemon = self.imp().daemon.get().unwrap();
        daemon.lock().unwrap().filesystem()
    }

    pub fn set_filesystem(&self, fs: BtrfsFilesystem) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        let updated = daemon.lock().unwrap().set_filesystem(fs)?;
        if updated {
            self.refresh_subvolumes();
        }
        Ok(())
    }

    pub fn delete_snapshot(&self, path: PathBuf) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        daemon.lock().unwrap().delete_subvolume(path)?;
        self.refresh_subvolumes();
        Ok(())
    }

    pub fn rename_snapshot(&self, before_path: PathBuf, after_path: PathBuf) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        daemon
            .lock()
            .unwrap()
            .move_subvolume(before_path, after_path)?;
        self.refresh_subvolumes();
        Ok(())
    }

    pub fn create_snapshot(
        &mut self,
        src: PathBuf,
        dest: PathBuf,
        readonly: bool,
    ) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        daemon.lock().unwrap().create_snapshot(
            src,
            dest,
            if readonly {
                libbtrfsutil::CreateSnapshotFlags::READ_ONLY.bits()
            } else {
                0
            },
        )?;
        self.refresh_subvolumes();
        Ok(())
    }

    pub fn is_schedule_enabled(&self) -> bool {
        let daemon = self.imp().daemon.get().unwrap();
        daemon.lock().unwrap().is_schedule_enabled()
    }

    pub fn set_is_schedule_enabled(&self, is_enabled: bool) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        daemon.lock().unwrap().set_is_schedule_enabled(is_enabled)?;
        Ok(())
    }

    pub fn schedules(&self) -> anyhow::Result<gio::ListStore> {
        let daemon = self.imp().daemon.get().unwrap();
        let schedules = daemon.lock().unwrap().schedules()?;
        let ret = gio::ListStore::new(BoxedAnyObject::static_type());
        for schedule in schedules {
            ret.append(&BoxedAnyObject::new(schedule));
        }
        Ok(ret)
    }

    pub fn fs_rename(&self, from: PathBuf, to: PathBuf) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        daemon.lock().unwrap().fs_rename(from, to)?;
        Ok(())
    }

    pub fn flush_schedule(&self, rule: JsonFile<Schedule>) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        daemon.lock().unwrap().flush_schedule(rule)?;
        Ok(())
    }
}
