mod name;
mod object;
pub use object::GRule;
use zbus::zvariant;

use std::{cmp, fs, io::{Result, ErrorKind, Read}, os::unix::prelude::OsStrExt, path::PathBuf};

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
    pub snapshot_on_battery: bool,
    pub prune_on_battery: bool,
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
    pub snapshot_on_battery: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub prune_on_battery: bool,
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
            snapshot_on_battery: e.snapshot_on_battery,
            prune_on_battery: e.prune_on_battery,
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
            snapshot_on_battery: e.snapshot_on_battery,
            prune_on_battery: e.prune_on_battery,
        }
    }
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

fn is_power_available() -> bool {
    let mut found_mains_online = false;
    let mut found_battery_not_discharging = false;
    for entry in fs::read_dir("/sys/class/power_supply").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let type_path = path.join("type");
            let online_path = path.join("online");
            let status_path = path.join("status");
            if type_path.exists() {
                let mut type_file = fs::File::open(type_path).unwrap();
                let mut type_contents = String::new();
                type_file.read_to_string(&mut type_contents).unwrap();
                if type_contents.trim() == "Mains" {
                    let mut online_file = fs::File::open(online_path).unwrap();
                    let mut online_contents = String::new();
                    online_file.read_to_string(&mut online_contents).unwrap();
                    if online_contents.trim() == "1" {
                        found_mains_online = true;
                        break;
                    }
                } else if type_contents.trim() == "Battery" {
                    let mut status_file = fs::File::open(status_path).unwrap();
                    let mut status_contents = String::new();
                    status_file.read_to_string(&mut status_contents).unwrap();
                    if status_contents.trim() != "Discharging" {
                        found_battery_not_discharging = true;
                        break;
                    }
                }
            }
        }
    }

    return found_mains_online || found_battery_not_discharging
}

impl Rule {
    pub fn open(path: PathBuf) -> Result<Self> {
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
        if !rule.snapshot_on_battery && !is_power_available() {
            log::info!("snapshot skipped because power is not available");
            return Ok(());
        }
        let mut name = RandomName::new();
        for _ in 0..16 {
            let target_path = self.target_dir.join(name.as_str());
            match create_butter_snapshot(&self.path, &target_path, true) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if e.kind() == ErrorKind::AlreadyExists {
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

        if !rule.prune_on_battery && !is_power_available() {
            log::info!("prune skipped because power is not available");
            return Ok(());
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
                println!("deleting '{}'", snapshot.path.display());
                let res = libbtrfsutil::DeleteSubvolumeOptions::new()
                    .recursive(true)
                    .delete(&snapshot.path);
                if let Err(err) = res {
                    eprintln!("failed to delete '{}': {}", snapshot.path.display(), err);
                }
            }
        }

        Ok(())
    }
}

pub struct ReadRuleDir(fs::ReadDir);

impl ReadRuleDir {
    pub fn new() -> Result<Self> {
        let conf_dir: PathBuf = PathBuf::from(config::PKGSYSCONFDIR).join("schedules");
        Ok(Self(fs::read_dir(conf_dir)?))
    }
}

impl Iterator for ReadRuleDir {
    type Item = Result<Rule>;

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
