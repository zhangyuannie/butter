use std::{
    fs, io,
    os::unix::prelude::*,
    path::{Path, PathBuf},
};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::schedule::select::{select_snapshots_to_remove, DateTimeSortable};

use self::{
    conf::{ReadScheduleDir, Schedule},
    name::RandomName,
};

mod conf;
mod name;
mod select;

pub fn cmd_snapshot() {
    for schedule in ReadScheduleDir::new().expect("Failed to read config directory") {
        if let Ok(schedule) = schedule {
            for subvol in &schedule.as_data().subvolumes {
                let mut name = RandomName::new();
                let mut has_err = true;
                for _ in 0..16 {
                    let target_path = subvol.target_dir.join(name.as_str());
                    match libbtrfsutil::create_snapshot(
                        &subvol.path,
                        target_path,
                        libbtrfsutil::CreateSnapshotFlags::READ_ONLY,
                        None,
                    ) {
                        Ok(_) => {
                            has_err = false;
                            break;
                        }
                        Err(e) => {
                            if e.os_error().kind() == io::ErrorKind::AlreadyExists {
                                name.inc_len();
                                continue;
                            } else {
                                break;
                            }
                        }
                    }
                }
                if has_err {
                    println!("Failed to create a snapshot for {}", subvol.path.display());
                }
            }
        }
    }
}

pub fn cmd_cleanup() {
    for schedule in ReadScheduleDir::new().expect("Failed to read config directory") {
        if let Ok(schedule) = schedule {
            let config = schedule.as_data();
            if !config.should_cleanup {
                continue;
            }
            for subvol in &config.subvolumes {
                let res = cleanup_single(config, &subvol.path, &subvol.target_dir);
                if let Err(err) = res {
                    println!("Failed to cleanup {}: {}", subvol.target_dir.display(), err);
                }
            }
        }
    }
}

fn cleanup_single(schedule: &Schedule, subvol: &Path, snapshot_dir: &Path) -> anyhow::Result<()> {
    struct Snapshot {
        path: PathBuf,
        created: NaiveDateTime,
    }
    impl DateTimeSortable for Snapshot {
        fn created(&self) -> &NaiveDateTime {
            &self.created
        }
    }
    let parent_uuid = libbtrfsutil::subvolume_info(subvol)?.uuid();
    let snapshots = fs::read_dir(snapshot_dir)?.filter_map(|entry| {
        let entry = entry.ok()?;
        if entry.file_name().as_bytes()[0] == b'.' {
            return None;
        }
        let path = entry.path();
        let info = libbtrfsutil::subvolume_info(&path).ok()?;

        if info.parent_uuid()? != parent_uuid {
            None
        } else {
            Some(Snapshot {
                path,
                created: DateTime::<Utc>::from(info.created()).naive_local(),
            })
        }
    });

    let to_remove = select_snapshots_to_remove(snapshots, schedule);

    for snapshot in to_remove {
        let res = libbtrfsutil::delete_subvolume(
            &snapshot.path,
            libbtrfsutil::DeleteSubvolumeFlags::RECURSIVE,
        );
        if let Err(err) = res {
            println!("Failed to delete {}: {}", snapshot.path.display(), err);
        }
    }

    Ok(())
}
