use crate::subvolume::{Subvolume, SubvolumeData};

use glib::once_cell::sync::OnceCell;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::io::{BufRead, BufReader, Write};
use std::process::{self, ChildStdin, ChildStdout};
use std::result;
use std::sync::Mutex;

mod daemon {

    use super::*;

    #[derive(Debug)]
    pub struct Daemon {
        pub reader: BufReader<ChildStdout>,
        pub writer: ChildStdin,
    }

    impl Daemon {
        pub fn run(&mut self, args: &[&str]) -> String {
            let req = serde_json::to_string(args).unwrap();
            writeln!(self.writer, "{}", req).unwrap();
            let mut reply = String::new();
            let byte_count = self.reader.read_line(&mut reply).unwrap();
            if byte_count == 0 {
                println!("Daemon exited unexpectedly!");
                process::exit(1);
            }
            reply
        }

        pub fn subvolumes(&mut self) -> Vec<SubvolumeData> {
            let reply_json = self.run(&["list_subvolumes"]);
            serde_json::from_str(&reply_json).unwrap()
        }

        pub fn rename_snapshot(&mut self, before: &str, after: &str) -> Option<String> {
            let reply_json = self.run(&["rename_snapshot", before, after]);
            serde_json::from_str(&reply_json).unwrap()
        }

        pub fn delete_snapshot(&mut self, path: &str) -> bool {
            let reply_json = self.run(&["delete_snapshot", path]);
            serde_json::from_str(&reply_json).unwrap()
        }

        pub fn create_snapshot(&mut self, src: &str, dest: &str, readonly: bool) -> bool {
            let ro_str = if readonly { "yes" } else { "" };
            let reply_json = self.run(&["create_snapshot", src, dest, ro_str]);
            serde_json::from_str(&reply_json).unwrap()
        }
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct SubvolumeManager {
        pub daemon: OnceCell<Mutex<daemon::Daemon>>,
        pub model: OnceCell<gio::ListStore>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubvolumeManager {
        const NAME: &'static str = "SubvolumeManager";
        type Type = super::SubvolumeManager;
    }

    impl ObjectImpl for SubvolumeManager {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            let model = gio::ListStore::new(Subvolume::static_type());
            self.model.set(model).expect("Failed to set model");
        }
    }
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

    pub fn model(&self) -> &gio::ListStore {
        self.imp().model.get().expect("Failed to get model")
    }

    pub fn refresh(&self) {
        let model = self.model();
        model.remove_all();
        let daemon = self.imp().daemon.get().unwrap();
        let subvols = daemon.lock().unwrap().subvolumes();
        for subvol in subvols {
            model.append(&Subvolume::from(subvol));
        }
    }

    pub fn delete_snapshot(&self, mounted_path: &str) -> bool {
        let daemon = self.imp().daemon.get().unwrap();
        let ret = daemon.lock().unwrap().delete_snapshot(mounted_path);
        self.refresh();
        ret
    }

    pub fn rename_snapshot(
        &self,
        before_path: &str,
        after_path: &str,
    ) -> result::Result<(), String> {
        let daemon = self.imp().daemon.get().unwrap();
        let ret = daemon
            .lock()
            .unwrap()
            .rename_snapshot(before_path, after_path);
        match ret {
            Some(error) => Err(error),
            None => {
                self.refresh();
                Ok(())
            }
        }
    }

    pub fn create_snapshot(&mut self, src: &str, dest: &str, readonly: bool) -> bool {
        let daemon = self.imp().daemon.get().unwrap();
        let ret = daemon.lock().unwrap().create_snapshot(src, dest, readonly);
        self.refresh();
        ret
    }
}
