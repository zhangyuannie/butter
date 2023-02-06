use std::{
    collections::HashMap,
    num::NonZeroU64,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use libbtrfsutil::{SubvolumeInfo, SubvolumeInfoIterator};
use once_cell::sync::OnceCell;
use zbus::zvariant::Optional;

use crate::subvolume::Subvolume;

use super::Filesystem;

pub trait SubvolumeExt {
    fn subvolumes(&self) -> Result<Vec<Subvolume>>;
}

struct IncompleteSubvolume {
    info: SubvolumeInfo,
    subvol_path: PathBuf,
    mnt_path_lazy: OnceCell<Option<PathBuf>>,
    is_mountpoint: bool,
}

impl IncompleteSubvolume {
    pub fn mnt_path(&self, subvol_by_id: &HashMap<u64, Self>) -> Option<&PathBuf> {
        self.mnt_path_lazy
            .get_or_init(|| {
                let parent_id = self.info.parent_id()?.get();
                let parent = subvol_by_id.get(&parent_id)?;
                let p_mnt_path = parent.mnt_path(subvol_by_id)?;
                let relative_path = self.subvol_path.strip_prefix(&parent.subvol_path).unwrap();
                Some(p_mnt_path.join(relative_path))
            })
            .as_ref()
    }
}

impl SubvolumeExt for Filesystem {
    fn subvolumes(&self) -> Result<Vec<Subvolume>> {
        // get an arbitary mount path of the filesystem
        let mnt_path = &self.mounts.values().next().unwrap()[0];
        let mut subvol_by_id = HashMap::new();

        // insert top level root subvolume
        {
            let info =
                libbtrfsutil::subvolume_info_with_id(mnt_path, libbtrfsutil::FS_TREE_OBJECTID)
                    .context("failed to get top-level subvol info")?;
            let mnt_path = self
                .mounts
                .get(&libbtrfsutil::FS_TREE_OBJECTID)
                .and_then(|vec| Some(PathBuf::from(&vec[0])));

            subvol_by_id.insert(
                info.id(),
                IncompleteSubvolume {
                    is_mountpoint: self.mounts.contains_key(&libbtrfsutil::FS_TREE_OBJECTID),
                    info,
                    subvol_path: "/".into(),
                    mnt_path_lazy: OnceCell::from(mnt_path),
                },
            );
        }

        // insert all other subvolumes
        {
            let root = Path::new("/");
            let iter = SubvolumeInfoIterator::new(
                mnt_path,
                NonZeroU64::new(libbtrfsutil::FS_TREE_OBJECTID),
                libbtrfsutil::SubvolumeIteratorFlags::empty(),
            )
            .context("failed to enumerate subvolumes")?;
            for e in iter {
                let (path, info) = e.unwrap();

                let id = info.id();
                subvol_by_id.insert(
                    id,
                    IncompleteSubvolume {
                        is_mountpoint: self.mounts.contains_key(&id),
                        info,
                        subvol_path: root.join(path),
                        mnt_path_lazy: if let Some(paths) = self.mounts.get(&id) {
                            OnceCell::from(Some(PathBuf::from(&paths[0])))
                        } else {
                            OnceCell::new()
                        },
                    },
                );
            }
        }

        let ret = subvol_by_id
            .values()
            .map(|subvol| Subvolume {
                is_mountpoint: subvol.is_mountpoint,
                subvol_path: subvol.subvol_path.to_owned(),
                mount_path: Optional::from(
                    subvol
                        .mnt_path(&subvol_by_id)
                        .and_then(|p| Some(p.to_owned())),
                ),
                uuid: subvol.info.uuid(),
                id: subvol.info.id(),
                created_unix_secs: subvol.info.otime(),
                snapshot_source_uuid: Optional::from(subvol.info.parent_uuid()),
            })
            .collect();

        Ok(ret)
    }
}
