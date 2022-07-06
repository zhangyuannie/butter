use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct ScheduleConf {
    schedules: Vec<Schedule>,
}

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
    pub subvols: Vec<ScheduleSubvolume>,
}

#[derive(Serialize, Deserialize)]
pub struct ScheduleSubvolume {
    source: PathBuf,
    target: PathBuf,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}
