use crate::subvolume::{GSubvolume, SubvolList};

#[allow(unused_imports)]
use gtk::prelude::*;

use butter::daemon::interface::DaemonInterface;
use glib::once_cell::sync::OnceCell;
use gtk::{glib, subclass::prelude::*};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{self, ChildStdin, ChildStdout};
use std::sync::Mutex;
use uuid::Uuid;

mod daemon {

    use butter::daemon::interface::{BtrfsFilesystem, DaemonInterface, Request, Result, Subvolume};

    use super::*;

    #[derive(Debug)]
    pub struct Daemon {
        pub reader: BufReader<ChildStdout>,
        pub writer: ChildStdin,
    }

    impl Daemon {
        pub fn run(&mut self, request: Request) -> Vec<u8> {
            let req = serde_json::to_string(&request).unwrap();
            writeln!(self.writer, "{}", req).unwrap();
            let mut ret = Vec::new();
            let byte_count = self.reader.read_until('\n' as u8, &mut ret).unwrap();
            if byte_count == 0 {
                println!("Daemon exited unexpectedly!");
                process::exit(1);
            }
            ret
        }
    }

    impl DaemonInterface for Daemon {
        fn list_filesystems(&mut self) -> Result<Vec<BtrfsFilesystem>> {
            serde_json::from_slice(&self.run(Request::ListFilesystems)).unwrap()
        }

        fn filesystem(&mut self) -> Option<String> {
            serde_json::from_slice(&self.run(Request::Filesystems)).unwrap()
        }

        fn set_filesystem(&mut self, device: BtrfsFilesystem) -> Result<()> {
            serde_json::from_slice(&self.run(Request::SetFilesystem(device))).unwrap()
        }

        fn list_subvolumes(&mut self) -> Result<Vec<Subvolume>> {
            serde_json::from_slice(&self.run(Request::ListSubvolumes)).unwrap()
        }

        fn move_subvolume(&mut self, from: PathBuf, to: PathBuf) -> Result<()> {
            serde_json::from_slice(&self.run(Request::MoveSubvolume(from, to))).unwrap()
        }

        fn delete_subvolume(&mut self, path: PathBuf) -> Result<()> {
            serde_json::from_slice(&self.run(Request::DeleteSubvolume(path))).unwrap()
        }

        fn create_snapshot(
            &mut self,
            src: PathBuf,
            dest: PathBuf,
            flags: libbtrfsutil::CreateSnapshotFlags,
        ) -> Result<Subvolume> {
            serde_json::from_slice(&self.run(Request::CreateSnapshot(src, dest, flags.bits())))
                .unwrap()
        }
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct SubvolumeManager {
        pub daemon: OnceCell<Mutex<daemon::Daemon>>,
        pub model: SubvolList,
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
            .set(Mutex::new(daemon::Daemon {
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
        let model = self.model();
        model.clear();
        let daemon = self.imp().daemon.get().unwrap();
        // TODO: smarter way to select the filesystem
        let fs_vec = daemon.lock().unwrap().list_filesystems().unwrap();
        daemon
            .lock()
            .unwrap()
            .set_filesystem(fs_vec.get(0).unwrap().clone())
            .unwrap();

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

        for (_, subvol) in subvols {
            model.insert(subvol);
        }
    }

    pub fn delete_snapshot(&self, path: PathBuf) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        daemon.lock().unwrap().delete_subvolume(path)?;
        self.refresh();
        Ok(())
    }

    pub fn rename_snapshot(&self, before_path: PathBuf, after_path: PathBuf) -> anyhow::Result<()> {
        let daemon = self.imp().daemon.get().unwrap();
        daemon
            .lock()
            .unwrap()
            .move_subvolume(before_path, after_path)?;
        self.refresh();
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
                libbtrfsutil::CreateSnapshotFlags::READ_ONLY
            } else {
                libbtrfsutil::CreateSnapshotFlags::empty()
            },
        )?;
        self.refresh();
        Ok(())
    }
}
