use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Butter specific metadata for snapshot
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SnapshotMetadata {
    // the path relative to the filesystem root of the subvolume this subvolume is a snapshot of
    pub created_from: PathBuf,
}
