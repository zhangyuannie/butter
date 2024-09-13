use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use zbus::zvariant;

use crate::config;

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, zvariant::Type)]
pub struct RuleConfig {
    pub is_enabled: bool,
    pub keep_hourly: u32,
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
    pub keep_yearly: u32,
    pub subvolumes: Vec<RuleSubvolumeConfig>,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, zvariant::Type)]
pub struct RuleSubvolumeConfig {
    pub path: PathBuf,
    pub target_dir: PathBuf,
}

impl RuleConfig {
    pub fn path(name: &str) -> PathBuf {
        Path::new(config::SCHEDULE_DIR).join(name)
    }
    pub fn read(name: &str) -> io::Result<Self> {
        let path = Self::path(name);

        let bytes = fs::read(&path)?;
        let mut de = serde_json::Deserializer::from_slice(&bytes);

        Ok(json::RuleConfig::deserialize(&mut de)?)
    }

    pub fn write(&self, name: &str, create_new: bool) -> io::Result<()> {
        let path = Self::path(name);

        let mut f = std::io::BufWriter::new(if create_new {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)?
        } else {
            fs::File::create(path)?
        });
        let mut ser = serde_json::Serializer::pretty(&mut f);
        json::RuleConfig::serialize(self, &mut ser)?;
        f.write_all(b"\n")?;
        f.flush()
    }
}

pub struct ReadScheduleDir(std::fs::ReadDir);

impl ReadScheduleDir {
    pub fn new() -> io::Result<Self> {
        Ok(Self(std::fs::read_dir(config::SCHEDULE_DIR)?))
    }
}

impl Iterator for ReadScheduleDir {
    type Item = io::Result<(String, RuleConfig)>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.0.next()?;
        match inner {
            Ok(entry) => {
                let Ok(name) = entry.file_name().into_string() else {
                    return Some(Err(io::ErrorKind::InvalidData.into()));
                };
                Some(RuleConfig::read(&name).map(|cfg| (name, cfg)))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

mod json {
    use super::*;

    fn is_default<T: Default + PartialEq>(t: &T) -> bool {
        t == &T::default()
    }

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "super::RuleConfig")]
    pub struct RuleConfig {
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
        pub subvolumes: Vec<RuleSubvolumeConfig>,
    }
}
