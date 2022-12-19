mod object;
pub use object::GFilesystem;
mod subvolume_ext;
pub use subvolume_ext::SubvolumeExt;

use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::zvariant::Type;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Type)]
pub struct Filesystem {
    pub label: String,
    pub uuid: Uuid,
    pub devices: Vec<PathBuf>,
    /// mount dirs by subvol ID
    pub mounts: HashMap<u64, Vec<PathBuf>>,
}
