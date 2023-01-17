use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Butter specific metadata for snapshot
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
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
        let metadata_bytes = fs::read(&metadata_path).ok()?;
        let ret: SnapshotMetadata = serde_json::from_slice(&metadata_bytes).ok()?;
        if ret.uuid == raw.uuid() {
            Some(ret)
        } else {
            None
        }
    }
}
