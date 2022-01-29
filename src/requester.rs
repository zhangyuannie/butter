use std::{
    io::{BufRead, BufReader, Write},
    process::{ChildStdin, ChildStdout},
    sync::{Mutex, MutexGuard},
};

use gtk::glib::once_cell::sync::Lazy;
use regex::Regex;

use crate::snapshot_object::SnapshotData;

#[derive(Debug, Default)]
pub struct Requester {
    reader: Option<BufReader<ChildStdout>>,
    writer: Option<ChildStdin>,
}

impl Requester {
    pub fn set_reader(&mut self, reader: BufReader<ChildStdout>) {
        self.reader = Some(reader);
    }

    pub fn set_writer(&mut self, writer: ChildStdin) {
        self.writer = Some(writer);
    }

    pub fn run_btrfs(&mut self, args: &[&str]) -> (i32, String) {
        let req = serde_json::to_string(args).unwrap();
        writeln!(self.writer.as_ref().unwrap(), "{}", req).unwrap();
        let mut reply = String::new();
        self.reader.as_mut().unwrap().read_line(&mut reply).unwrap();
        serde_json::from_str(&reply).unwrap()
    }

    pub fn snapshots(&mut self) -> Vec<SnapshotData> {
        let (status, s) = self.run_btrfs(&["subvolume", "list", "-sq", "/"]);
        assert_eq!(status, 0);

        let re = Regex::new(r"otime (.*) parent_uuid (\S*) .*path (.*)").unwrap();
        let mut ret = Vec::new();
        for line in s.lines() {
            let captures = re.captures(line).unwrap();
            ret.push(SnapshotData {
                parent_path: self.uuid_to_path(captures.get(2).unwrap().as_str()),
                creation_time: captures.get(1).unwrap().as_str().into(),
                path: captures.get(3).unwrap().as_str().into(),
            })
        }
        ret
    }

    fn uuid_to_path(&mut self, uuid: &str) -> String {
        let (status, s) = self.run_btrfs(&["subvolume", "list", "-u", "/"]);
        assert_eq!(status, 0);

        let re = Regex::new(r"uuid (\S*) path (.*)").unwrap();
        for line in s.lines() {
            let captures = re.captures(line).unwrap();
            if captures.get(1).unwrap().as_str() == uuid {
                return captures.get(2).unwrap().as_str().into();
            }
        }
        "Unknown".into()
    }
}

static DAEMON: Lazy<Mutex<Requester>> = Lazy::new(|| Mutex::default());

pub fn daemon() -> MutexGuard<'static, Requester> {
    DAEMON.lock().unwrap()
}
