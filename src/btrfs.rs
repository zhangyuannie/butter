use regex::Regex;
use std::process::Command;
pub struct Snapshot {
    path: String,
    parent_path: String,
    creation_time: String,
}

pub fn snapshots() -> Vec<Snapshot> {
    let res = Command::new("btrfs")
        .args(["subvolume", "list", "-sq", "/"])
        .output()
        .unwrap();
    assert!(res.status.success());
    let s = String::from_utf8(res.stdout).unwrap();
    let re = Regex::new(r"otime (.*) parent_uuid (\S*) .*path (.*)").unwrap();
    let mut ret = Vec::new();
    for line in s.split("\n") {
        let captures = re.captures(line).unwrap();
        ret.push(Snapshot {
            parent_path: uuid_to_path(captures.get(2).unwrap().as_str()),
            creation_time: captures.get(1).unwrap().as_str().into(),
            path: captures.get(3).unwrap().as_str().into(),
        })
    }
    ret
}

fn uuid_to_path(uuid: &str) -> String {
    let res = Command::new("btrfs")
        .args(["subvolume", "list", "-u", "/"])
        .output()
        .unwrap();
    assert!(res.status.success());
    let s = String::from_utf8(res.stdout).unwrap();
    let re = Regex::new(r"uuid (\S*) path (.*)").unwrap();
    for line in s.split("\n") {
        let captures = re.captures(line).unwrap();
        if captures.get(1).unwrap().as_str() == uuid {
            return captures.get(2).unwrap().as_str().into();
        }
    }
    "Unknown".into()
}
