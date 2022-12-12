pub use zbus;

use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::zvariant::{Optional, Type};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Type)]
pub struct BtrfsFilesystem {
    pub label: String,
    pub uuid: Uuid,
    pub devices: Vec<PathBuf>,
    /// mount dirs by subvol ID
    pub mounts: HashMap<u64, Vec<PathBuf>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Type)]
pub struct Subvolume {
    pub subvol_path: PathBuf,
    pub mount_path: Optional<PathBuf>,
    pub uuid: Uuid,
    pub id: u64,
    pub created_unix_secs: i64,
    pub snapshot_source_uuid: Optional<Uuid>,
}

impl Default for Subvolume {
    fn default() -> Self {
        Self {
            subvol_path: Default::default(),
            mount_path: Optional::from(None),
            uuid: Default::default(),
            id: Default::default(),
            created_unix_secs: Default::default(),
            snapshot_source_uuid: Optional::from(None),
        }
    }
}
