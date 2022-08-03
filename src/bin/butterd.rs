use std::fs;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use anyhow::Context;
use butter::daemon::cmd;
use butter::daemon::interface::{BtrfsFilesystem, DaemonInterface, Result, Subvolume};
use butter::daemon::mounted_fs::MountedTopLevelSubvolume;
use butter::json_file::JsonFile;
use butter::schedule::{ReadScheduleDir, Schedule};
use libbtrfsutil::CreateSnapshotFlags;
use libc::c_int;
use uuid::Uuid;

struct MountedFs {
    info: BtrfsFilesystem,
    subvol: MountedTopLevelSubvolume,
}

#[derive(Default)]
struct Daemon {
    fs: Option<MountedFs>,
}

impl Daemon {
    fn new() -> Daemon {
        Daemon::default()
    }

    fn path_within_fs<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        Ok(self
            .fs
            .as_ref()
            .context("no filesystem set")?
            .subvol
            .path()
            .join(path))
    }
}

impl DaemonInterface for Daemon {
    fn list_filesystems(&mut self) -> Result<Vec<BtrfsFilesystem>> {
        let ret = cmd::btrfs_filesystem_show()?;
        Ok(ret)
    }

    fn filesystem(&mut self) -> Option<Uuid> {
        self.fs.as_ref().and_then(|fs| Some(fs.info.uuid))
    }

    fn set_filesystem(&mut self, fs: BtrfsFilesystem) -> Result<bool> {
        if let Some(cur_fs) = &self.fs {
            if cur_fs.info.uuid == fs.uuid {
                return Ok(false);
            }
        }
        self.fs = Some(MountedFs {
            subvol: MountedTopLevelSubvolume::new(
                fs.devices
                    .get(0)
                    .context("filesystem has empty device list")?,
            )?,
            info: fs,
        });

        Ok(true)
    }

    fn list_subvolumes(&mut self) -> Result<Vec<Subvolume>> {
        let mount_path = self.fs.as_ref().context("no filesystem set")?.subvol.path();

        let toplevel = libbtrfsutil::subvolume_info(mount_path)
            .context("failed to get top-level subvol info")?;

        let mut ret = vec![Subvolume {
            path: PathBuf::from("/"),
            uuid: toplevel.uuid(),
            created: toplevel.created(),
            snapshot_source_uuid: toplevel.parent_uuid(),
        }];

        ret.extend(
            libbtrfsutil::SubvolumeInfoIterator::new(
                mount_path,
                NonZeroU64::new(libbtrfsutil::FS_TREE_OBJECTID),
                libbtrfsutil::SubvolumeIteratorFlags::empty(),
            )
            .context("failed to enumerate subvolumes")?
            .map(|subvol| {
                let (path, info) = subvol.unwrap();
                Subvolume {
                    path,
                    uuid: info.uuid(),
                    created: info.created(),
                    snapshot_source_uuid: info.parent_uuid(),
                }
            }),
        );

        Ok(ret)
    }

    fn move_subvolume(&mut self, from: PathBuf, to: PathBuf) -> Result<()> {
        fs::rename(self.path_within_fs(from)?, self.path_within_fs(to)?)
            .context("failed to move subvolume")?;
        Ok(())
    }

    fn delete_subvolume(&mut self, path: PathBuf) -> Result<()> {
        libbtrfsutil::delete_subvolume(
            self.path_within_fs(path)?,
            libbtrfsutil::DeleteSubvolumeFlags::RECURSIVE,
        )
        .context("failed to delete subvolume")?;
        Ok(())
    }

    fn create_snapshot(&mut self, src: PathBuf, dest: PathBuf, flags: c_int) -> Result<Subvolume> {
        if let Some(dest_parent) = dest.parent() {
            fs::create_dir_all(dest_parent).context("failed to create target parent")?;
        }

        libbtrfsutil::create_snapshot(
            self.path_within_fs(&src)?,
            self.path_within_fs(&dest)?,
            CreateSnapshotFlags::from_bits_truncate(flags),
            None,
        )
        .context("failed to create snapshot")?;

        let info = libbtrfsutil::subvolume_info(&dest).context("failed to get snapshot info")?;

        Ok(Subvolume {
            path: dest,
            uuid: info.uuid(),
            created: info.created(),
            snapshot_source_uuid: info.parent_uuid(),
        })
    }

    fn is_schedule_enabled(&mut self) -> bool {
        cmd::is_systemd_unit_active("butter-schedule-snapshot.timer").unwrap_or(false)
    }

    fn set_is_schedule_enabled(&mut self, is_enabled: bool) -> Result<()> {
        if is_enabled {
            cmd::enable_systemd_unit("butter-schedule-snapshot.timer", true)?;
            cmd::enable_systemd_unit("butter-schedule-prune.timer", true)?;
        } else {
            cmd::disable_systemd_unit("butter-schedule-snapshot.timer", true)?;
            cmd::disable_systemd_unit("butter-schedule-prune.timer", true)?;
        }
        Ok(())
    }

    fn schedules(&mut self) -> Result<Vec<JsonFile<Schedule>>> {
        let schedules = ReadScheduleDir::new().context("Failed to read config directory")?;
        Ok(schedules.map_while(|s| s.ok()).collect())
    }
}

fn main() {
    let mut d = Daemon::new();
    d.serve();
}
