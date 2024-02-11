mod name;
mod object;
pub use object::GRule;
use zbus::zvariant;

use std::{cmp, fs, io, os::unix::prelude::OsStrExt, path::PathBuf, ffi::OsStr};

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use log;
use serde::{Deserialize, Serialize};

use crate::{config, daemon::btrfs::create_butter_snapshot, subvolume::SnapshotMetadata};

use self::name::RandomName;

#[derive(Debug, Default, Clone, Serialize, Deserialize, zvariant::Type)]
pub struct Rule {
    pub path: PathBuf,
    pub is_enabled: bool,
    pub keep_hourly: u32,
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
    pub keep_yearly: u32,
    pub subvolumes: Vec<RuleSubvolume>,
    pub pre_snapshot: String,
    pub post_snapshot: String,
    pub pre_prune: String,
    pub post_prune: String,
}

#[derive(Serialize, Deserialize)]
pub struct RuleJson {
    #[serde(skip_serializing, skip_deserializing)]
    pub path: PathBuf,
    pub is_enabled: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub keep_hourly: u32,
    #[serde(default, skip_serializing_if = "is_default")]
    pub keep_daily: u32,
    #[serde(default, skip_serializing_if = "is_default")]
    pub keep_weekly: u32,
    #[serde(default, skip_serializing_if = "is_default")]
    pub keep_monthly: u32,
    #[serde(default, skip_serializing_if = "is_default")]
    pub keep_yearly: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subvolumes: Vec<RuleSubvolume>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub pre_snapshot: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub post_snapshot: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub pre_prune: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub post_prune: String,
}

impl From<Rule> for RuleJson {
    fn from(e: Rule) -> Self {
        Self {
            path: e.path,
            is_enabled: e.is_enabled,
            keep_hourly: e.keep_hourly,
            keep_daily: e.keep_daily,
            keep_weekly: e.keep_weekly,
            keep_monthly: e.keep_monthly,
            keep_yearly: e.keep_yearly,
            subvolumes: e.subvolumes,
            pre_snapshot: e.pre_snapshot,
            post_snapshot: e.post_snapshot,
            pre_prune: e.pre_prune,
            post_prune: e.post_prune,
        }
    }
}

impl From<RuleJson> for Rule {
    fn from(e: RuleJson) -> Self {
        Self {
            path: e.path,
            is_enabled: e.is_enabled,
            keep_hourly: e.keep_hourly,
            keep_daily: e.keep_daily,
            keep_weekly: e.keep_weekly,
            keep_monthly: e.keep_monthly,
            keep_yearly: e.keep_yearly,
            subvolumes: e.subvolumes,
            pre_snapshot: e.pre_snapshot,
            post_snapshot: e.post_snapshot,
            pre_prune: e.pre_prune,
            post_prune: e.post_prune,
        }
    }
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

impl Rule {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let bytes = fs::read(&path)?;
        let mut data: RuleJson = serde_json::from_slice(&bytes)?;
        data.path = path;
        Ok(data.into())
    }

    pub fn should_prune(&self) -> bool {
        self.keep_hourly != 0
            || self.keep_daily != 0
            || self.keep_weekly != 0
            || self.keep_monthly != 0
            || self.keep_yearly != 0
    }

    pub fn snapshot(&self) {
        for subvol in &self.subvolumes {
            log::info!(
                "creating a snapshot from '{}' in '{}'",
                subvol.path.display(),
                subvol.target_dir.display()
            );
            let ret = subvol.snapshot(self);
            if let Err(e) = ret {
                log::error!(
                    "failed to create a snapshot from '{}': {}",
                    subvol.path.display(),
                    e
                );
            }
        }
    }

    pub fn prune(&self) {
        if !self.should_prune() {
            return;
        }

        for subvol in &self.subvolumes {
            let res = subvol.prune(self);
            if let Err(err) = res {
                eprintln!("failed to prune '{}': {}", subvol.target_dir.display(), err);
            }
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, zvariant::Type)]
pub struct RuleSubvolume {
    pub path: PathBuf,
    pub target_dir: PathBuf,
}

impl RuleSubvolume {
    fn snapshot(&self, rule: &Rule) -> anyhow::Result<()> {
        let mut name = RandomName::new();
        for _ in 0..16 {
            let target_path = self.target_dir.join(name.as_str());
            if !rule.pre_snapshot.is_empty() {
                let mut cmd = std::process::Command::new("sh");
                cmd.arg("-c").arg(format!("{} {} {}", rule.pre_snapshot, self.path.display(), target_path.display()));
                let res = cmd.output();
                match res {
                    Ok(output) => {
                        if !output.status.success() {
                            let error_code = output.status.code();
                            if let Some(code) = error_code {
                                eprintln!("Pre Snapshot script failed, aborting snapshot, exit code: {}, error: {}", code, String::from_utf8_lossy(&output.stderr));
                            } else {
                                eprintln!("Pre Snapshot script failed, aborting snapshot, error: {}", String::from_utf8_lossy(&output.stderr));
                            }
                            return Err(anyhow::anyhow!("Pre snapshot script failed"));
                        }
                    }
                    Err(e) => {
                        eprintln!("failed to run Pre Snapshot script: {}", e);
                        return Err(e.into());
                    }
                }
            }
            match create_butter_snapshot(&self.path, &target_path, true) {
                Ok(_) => {
                    if !rule.post_snapshot.is_empty() {
                        let mut cmd = std::process::Command::new("sh");
                        cmd.arg("-c").arg(format!("{} {} {}", rule.post_snapshot, self.path.display(), target_path.display()));
                        let res = cmd.output();
                        if let Err(e) = res {
                            eprintln!("failed to run Post Snapshot script: {}", e);
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::AlreadyExists {
                        name.inc_len();
                        continue;
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
        Err(anyhow::anyhow!("name exhausted"))
    }

    fn prune(&self, rule: &Rule) -> anyhow::Result<()> {
        #[derive(Debug, PartialEq, Eq)]
        struct Snapshot {
            created: NaiveDateTime,
            path: PathBuf,
        }
        struct Bucket {
            keep: u32,
            last: i32,
            algo: fn(&NaiveDateTime) -> i32,
        }

        let source_subvol_path = libbtrfsutil::subvolume_path(&self.path)?;

        let mut snapshots: Vec<Snapshot> = fs::read_dir(&self.target_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_name().as_bytes()[0] == b'.' {
                    return None;
                }
                let path = entry.path();
                let info = libbtrfsutil::subvolume_info(&path).ok()?;
                if let Some(metadata) = SnapshotMetadata::read(&path) {
                    if metadata.created_from == source_subvol_path {
                        return Some(Snapshot {
                            path,
                            created: DateTime::<Utc>::from(info.created()).naive_local(),
                        });
                    }
                }
                return None;
            })
            .collect();
        snapshots.sort_by_key(|e| cmp::Reverse(e.created));

        let mut buckets = [
            Bucket {
                keep: rule.keep_hourly,
                last: 0,
                algo: |dt| dt.year() * 100000 + dt.ordinal() as i32 * 100 + dt.hour() as i32,
            },
            Bucket {
                keep: rule.keep_daily,
                last: 0,
                algo: |dt| dt.year() * 1000 + dt.ordinal() as i32,
            },
            Bucket {
                keep: rule.keep_weekly,
                last: 0,
                algo: |dt| {
                    let week = dt.iso_week();
                    week.year() * 100 + week.week() as i32
                },
            },
            Bucket {
                keep: rule.keep_monthly,
                last: 0,
                algo: |dt| dt.year() * 100 + dt.month() as i32,
            },
            Bucket {
                keep: rule.keep_yearly,
                last: 0,
                algo: |dt| dt.year(),
            },
        ];

        for snapshot in snapshots {
            let mut should_remove = true;
            for bucket in &mut buckets {
                if bucket.keep > 0 {
                    let val = (bucket.algo)(&snapshot.created);
                    if val != bucket.last {
                        bucket.keep -= 1;
                        bucket.last = val;
                        should_remove = false;
                    }
                }
            }

            if should_remove {
                if !rule.pre_prune.is_empty() {
                    let mut cmd = std::process::Command::new("sh");
                    cmd.arg("-c").arg(format!("{} {}", rule.pre_prune, snapshot.path.display()));
                    let res = cmd.output();
                    match res {
                        Ok(output) => {
                            if !output.status.success() {
                                let error_code = output.status.code();
                                if let Some(code) = error_code {
                                    eprintln!("Pre Prune script failed, aborting prune, exit code: {}, error: {}", code, String::from_utf8_lossy(&output.stderr));
                                } else {
                                    eprintln!("Pre Prune script failed, aborting prune, error: {}", String::from_utf8_lossy(&output.stderr));
                                }
                                return Err(anyhow::anyhow!("Pre Prune script failed"));
                            }
                        }
                        Err(e) => {
                            eprintln!("failed to run Pre Prune script: {}", e);
                            return Err(e.into());
                        }
                    }
                }
                println!("deleting '{}'", snapshot.path.display());
                let res = libbtrfsutil::DeleteSubvolumeOptions::new()
                    .recursive(true)
                    .delete(&snapshot.path);
                if let Err(err) = res {
                    eprintln!("failed to delete '{}': {}", snapshot.path.display(), err);
                } else {
                    if !rule.post_prune.is_empty() {
                        let mut cmd = std::process::Command::new("sh");
                        cmd.arg("-c").arg(format!("{} {}", rule.post_prune, snapshot.path.display()));
                        let res = cmd.output();
                        if let Err(e) = res {
                            eprintln!("failed to run Post Prune script: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct ReadRuleDir(fs::ReadDir);

impl ReadRuleDir {
    pub fn new() -> io::Result<Self> {
        let conf_dir: PathBuf = PathBuf::from(config::PKGSYSCONFDIR).join("schedules");
        Ok(Self(fs::read_dir(conf_dir)?))
    }
}

impl Iterator for ReadRuleDir {
    type Item = io::Result<Rule>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.0.next()?;
        match inner {
            Ok(entry) => {
                let path = entry.path();
                Some(Rule::open(path))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
