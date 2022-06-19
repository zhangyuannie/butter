use std::fs;
use std::io::{self, BufRead};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use anyhow::Context;
use butter::daemon::cmd::btrfs_filesystem_show;
use butter::daemon::interface::{BtrfsFilesystem, DaemonInterface, Request, Result, Subvolume};
use butter::daemon::mounted_fs::MountedTopLevelSubvolume;
use libbtrfsutil::CreateSnapshotFlags;
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
        let ret = btrfs_filesystem_show()?;
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
        Ok(libbtrfsutil::SubvolumeInfoIterator::new(
            self.fs.as_ref().context("no filesystem set")?.subvol.path(),
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
        })
        .collect())
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

    fn create_snapshot(
        &mut self,
        src: PathBuf,
        dest: PathBuf,
        flags: libbtrfsutil::CreateSnapshotFlags,
    ) -> Result<Subvolume> {
        if let Some(dest_parent) = dest.parent() {
            fs::create_dir_all(dest_parent).context("failed to create target parent")?;
        }

        libbtrfsutil::create_snapshot(
            self.path_within_fs(&src)?,
            self.path_within_fs(&dest)?,
            flags,
            None,
        )
        .context("failed to create snapshot")?;

        let info = libbtrfsutil::subvolume_info(dest.as_path(), None)
            .context("failed to get snapshot info")?;

        Ok(Subvolume {
            path: dest,
            uuid: info.uuid(),
            created: info.created(),
            snapshot_source_uuid: info.parent_uuid(),
        })
    }
}

fn main() {
    let mut d = Daemon::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }
        let req: Request = serde_json::from_str(&line).unwrap();
        eprintln!("{:?}", req);
        let response = match req {
            Request::ListFilesystems => serde_json::to_string(&d.list_filesystems()),
            Request::Filesystems => serde_json::to_string(&d.filesystem()),
            Request::SetFilesystem(device) => serde_json::to_string(&d.set_filesystem(device)),
            Request::ListSubvolumes => serde_json::to_string(&d.list_subvolumes()),
            Request::MoveSubvolume(from, to) => serde_json::to_string(&d.move_subvolume(from, to)),
            Request::DeleteSubvolume(path) => serde_json::to_string(&d.delete_subvolume(path)),
            Request::CreateSnapshot(src, dest, flags) => serde_json::to_string(&d.create_snapshot(
                src,
                dest,
                CreateSnapshotFlags::from_bits_truncate(flags),
            )),
        };
        let response = response.expect("failed to serialize response");

        println!("{}", response);
    }
}
