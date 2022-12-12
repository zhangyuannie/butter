use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use butter::daemon::interface::{Butterd, Result};
use butter::json_file::JsonFile;
use butter::schedule::{ReadScheduleDir, Schedule};

#[derive(Default)]
struct Daemon;

impl Daemon {
    fn new() -> Daemon {
        Daemon::default()
    }
}

impl Butterd for Daemon {
    fn schedules(&mut self) -> Result<Vec<JsonFile<Schedule>>> {
        let schedules = ReadScheduleDir::new().context("Failed to read config directory")?;
        Ok(schedules.map_while(|s| s.ok()).collect())
    }

    fn fs_rename(&mut self, from: PathBuf, to: PathBuf) -> Result<()> {
        Ok(fs::rename(from, to).context("fs_rename failed")?)
    }

    fn flush_schedule(&mut self, rule: JsonFile<Schedule>) -> Result<()> {
        Ok(rule.flush().context("flush_schedule failed")?)
    }

    fn fs_remove_file(&mut self, path: PathBuf) -> Result<()> {
        Ok(fs::remove_file(path).context("fs_remove_file failed")?)
    }
}

fn main() {
    let mut d = Daemon::new();
    d.serve();
}
