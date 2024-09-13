use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::zvariant::{Optional, Type};

use crate::{ZPathBuf, ZUuid};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Type)]
pub struct Subvolume {
    /// relative path from the root subvol
    pub root_path: ZPathBuf,
    /// root path of the subvolume this is a snapshot of at the time.
    ///
    /// This field is best effort and may not exist for snapshots created
    /// elsewhere.
    pub created_from_root_path: Option<ZPathBuf>,
    pub paths: Vec<ZPathBuf>,
    /// `true` if found in /proc/self/mounts
    pub is_mountpoint: bool,
    pub uuid: ZUuid,
    pub id: u64,
    pub created_unix_secs: i64,
    pub snapshot_source_uuid: Optional<ZUuid>,
}

impl Subvolume {
    /// `true` if it is used as a main subvolumne, not as a backup snapshot.
    ///
    /// What BTRFS technically considers as snapshot may be mounted as /home
    /// thus "acts" as a main subvolume.
    pub fn is_likely_primary(&self) -> bool {
        // Some hardcoded paths that are extremly often to be their own subvolume
        // and important for the system.
        static USUALS: LazyLock<HashSet<PathBuf>> = LazyLock::new(|| {
            HashSet::from([
                PathBuf::from("/"),
                PathBuf::from("/boot"),
                PathBuf::from("/home"),
                PathBuf::from("/mnt"),
                PathBuf::from("/opt"),
                PathBuf::from("/root"),
                PathBuf::from("/srv"),
                PathBuf::from("/tmp"),
                PathBuf::from("/usr"),
                PathBuf::from("/usr/local"),
                PathBuf::from("/var"),
                PathBuf::from("/var/lib"),
                PathBuf::from("/var/lib/machines"),
                PathBuf::from("/var/log"),
                PathBuf::from("/var"),
            ])
        });

        self.is_mountpoint
            || self.snapshot_source_uuid.is_none()
            || self.paths.iter().any(|p| USUALS.contains(p.as_path()))
    }
}

/// Butter specific metadata for snapshot
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SnapshotMetadata {
    /// the path relative to the filesystem root of the subvolume this subvolume is a snapshot of
    pub created_from: PathBuf,
    /// subvolume's UUID
    pub uuid: Uuid,
}

impl SnapshotMetadata {
    pub fn read(subvol_path: &Path) -> Option<SnapshotMetadata> {
        let raw = libbtrfsutil::subvolume_info(subvol_path).ok()?;
        let metadata_path = subvol_path.join(".butter/info.json");
        let metadata_bytes = std::fs::read(&metadata_path).ok()?;
        let ret: SnapshotMetadata = serde_json::from_slice(&metadata_bytes).ok()?;
        if ret.uuid == raw.uuid() {
            Some(ret)
        } else {
            None
        }
    }
}

/// Create a regular snapshot, save butter specific metadata, conditionally make it read-only
pub fn create_snapshot(src_path: &Path, dst_path: &Path, readonly: bool) -> io::Result<()> {
    let src_subvol_path = libbtrfsutil::subvolume_path(src_path).map_err(|e| e.os_error())?;
    if let Some(dst_parent) = dst_path.parent() {
        std::fs::create_dir_all(dst_parent)?;
    }
    libbtrfsutil::CreateSnapshotOptions::new()
        .create(src_path, dst_path)
        .map_err(|e| e.os_error())?;

    let metadata_dir = dst_path.join(".butter");
    std::fs::create_dir_all(&metadata_dir)?;

    let metadata = SnapshotMetadata {
        created_from: src_subvol_path,
        uuid: libbtrfsutil::subvolume_info(dst_path)
            .map_err(|e| e.os_error())?
            .uuid(),
    };

    let mut f = BufWriter::new(File::create(metadata_dir.join("info.json"))?);
    serde_json::to_writer_pretty(&mut f, &metadata)?;
    f.write_all(b"\n")?;
    f.flush()?;

    libbtrfsutil::set_subvolume_read_only(dst_path, readonly).map_err(|e| e.os_error())?;
    Ok(())
}
