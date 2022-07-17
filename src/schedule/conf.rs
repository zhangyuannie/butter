use serde::{Deserialize, Serialize};
use std::{
    fs::{self, ReadDir},
    io,
    path::PathBuf,
};

use crate::{config, JsonFile};

#[derive(Serialize, Deserialize)]
pub struct Schedule {
    pub label: String,
    pub is_enabled: bool,
    pub should_cleanup: bool,
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
    pub subvolumes: Vec<ScheduleSubvolume>,
}

#[derive(Serialize, Deserialize)]
pub struct ScheduleSubvolume {
    pub path: PathBuf,
    pub target_dir: PathBuf,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub struct ReadScheduleDir(ReadDir);

impl ReadScheduleDir {
    pub fn new() -> io::Result<Self> {
        let conf_dir = PathBuf::new().join(config::PKGSYSCONFDIR).join("schedules");
        Ok(ReadScheduleDir(fs::read_dir(conf_dir)?))
    }
}

impl Iterator for ReadScheduleDir {
    type Item = io::Result<JsonFile<Schedule>>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.0.next()?;
        match inner {
            Ok(entry) => {
                let path = entry.path();
                Some(JsonFile::open(path))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
