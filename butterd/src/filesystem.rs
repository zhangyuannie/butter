use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    path::PathBuf,
};

use anyhow::Context;
use uuid::Uuid;
use zbus::{interface, zvariant::ObjectPath};

use crate::{Subvolume, ToFdo, ZPathBuf, ZUuid};

pub struct Filesystem {
    pub(crate) uuid: ZUuid,
    pub(crate) label: String,
    /// Must be Sorted
    pub(crate) devices: Vec<ZPathBuf>,
    pub(crate) mount_points_by_subvol_id: HashMap<u64, Vec<ZPathBuf>>,
}

impl Filesystem {
    pub(crate) async fn update(
        self,
        server: &zbus::ObjectServer,
        path: &ObjectPath<'_>,
    ) -> anyhow::Result<()> {
        let iface_ref = server.interface::<_, Filesystem>(path).await?;
        let mut iface = iface_ref.get_mut().await;

        if iface.uuid != self.uuid {
            return Err(anyhow::anyhow!("uuid mismatch"));
        }

        if iface.label != self.label {
            iface.label = self.label;
            iface.label_changed(iface_ref.signal_context()).await?;
        }

        if iface.devices != self.devices {
            iface.devices = self.devices;
            iface.devices_changed(iface_ref.signal_context()).await?;
        }

        Ok(())
    }

    pub(crate) async fn create(
        self,
        server: &zbus::ObjectServer,
        path: &ObjectPath<'_>,
    ) -> anyhow::Result<()> {
        server.at(path, self).await?;
        Ok(())
    }

    fn list_subvolumes_impl(&self) -> anyhow::Result<Vec<Subvolume>> {
        struct PartialSubvol {
            info: libbtrfsutil::SubvolumeInfo,
            root_path: PathBuf,
            final_paths: OnceCell<Vec<ZPathBuf>>,
            start_paths: RefCell<Vec<ZPathBuf>>,
            is_mountpoint: bool,
        }
        impl PartialSubvol {
            fn compute_paths(&self, subvol_by_id: &HashMap<u64, Self>) -> Vec<ZPathBuf> {
                let mut ret = self.start_paths.take();

                if let Some((parent_root_path, parent_paths)) = self
                    .info
                    .parent_id()
                    .and_then(|parent_id| subvol_by_id.get(&parent_id.into()))
                    .map(|p| (&p.root_path, p.paths(subvol_by_id)))
                {
                    if let Ok(relative_path) = self.root_path.strip_prefix(parent_root_path) {
                        ret.extend(
                            parent_paths
                                .iter()
                                .map(|p| p.as_path().join(relative_path).into()),
                        )
                    }
                }

                ret.sort_unstable();
                ret.dedup();
                ret
            }
            fn paths(&self, subvol_by_id: &HashMap<u64, Self>) -> &[ZPathBuf] {
                self.final_paths
                    .get_or_init(|| self.compute_paths(subvol_by_id))
            }
            fn created_from_root_path(
                &self,
                subvol_by_uuid: &HashMap<Uuid, &PartialSubvol>,
            ) -> Option<ZPathBuf> {
                let created_from_uuid = self.info.parent_uuid()?;
                subvol_by_uuid
                    .get(&created_from_uuid)
                    .map(|subvol| subvol.root_path.clone().into())
            }
        }

        // get an arbitary mount path of the filesystem
        let mnt_path = self
            .mount_points_by_subvol_id
            .values()
            .next()
            .context("Filesystem must be mounted")?[0]
            .as_path();
        let mut subvol_by_id = HashMap::new();

        // insert top level root subvolume
        {
            let info =
                libbtrfsutil::subvolume_info_with_id(mnt_path, libbtrfsutil::FS_TREE_OBJECTID)
                    .context("failed to get top-level subvol info")?;
            let paths = self
                .mount_points_by_subvol_id
                .get(&libbtrfsutil::FS_TREE_OBJECTID)
                .cloned()
                .unwrap_or_default();

            subvol_by_id.insert(
                info.id(),
                PartialSubvol {
                    is_mountpoint: !paths.is_empty(),
                    info,
                    root_path: Default::default(),
                    final_paths: OnceCell::from(paths),
                    start_paths: Default::default(),
                },
            );
        }

        // insert all other subvolumes
        {
            let iter = libbtrfsutil::IterateSubvolume::new(mnt_path)
                .all()
                .iter_with_info()
                .context("failed to enumerate subvolumes")?;
            for (root_path, info) in iter.flatten() {
                let id = info.id();
                let start_paths = self
                    .mount_points_by_subvol_id
                    .get(&id)
                    .cloned()
                    .unwrap_or_default();
                subvol_by_id.insert(
                    id,
                    PartialSubvol {
                        is_mountpoint: !start_paths.is_empty(),
                        info,
                        root_path,
                        final_paths: Default::default(),
                        start_paths: start_paths.into(),
                    },
                );
            }
        }

        let subvol_by_uuid: HashMap<Uuid, &PartialSubvol> = subvol_by_id
            .values()
            .map(|item| (item.info.uuid(), item))
            .collect();

        let ret = subvol_by_id
            .values()
            .map(|subvol| Subvolume {
                root_path: subvol.root_path.clone().into(),
                created_from_root_path: subvol.created_from_root_path(&subvol_by_uuid),
                paths: subvol.paths(&subvol_by_id).to_owned(),
                is_mountpoint: subvol.is_mountpoint,
                uuid: subvol.info.uuid().into(),
                id: subvol.info.id(),
                created_unix_secs: subvol.info.otime(),
                snapshot_source_uuid: subvol.info.parent_uuid().map(Into::into).into(),
            })
            .collect();

        Ok(ret)
    }
}

#[interface(
    name = "org.zhangyuannie.Butter1.Filesystem",
    proxy(gen_blocking = true, default_service = "org.zhangyuannie.Butter1")
)]
impl Filesystem {
    #[zbus(property(emits_changed_signal = "const"))]
    fn uuid(&self) -> ZUuid {
        self.uuid
    }

    #[zbus(property)]
    fn label(&self) -> String {
        self.label.clone()
    }

    #[zbus(property)]
    fn devices(&self) -> Vec<ZPathBuf> {
        self.devices.clone()
    }

    fn list_subvolumes(&self) -> zbus::fdo::Result<Vec<Subvolume>> {
        self.list_subvolumes_impl().to_fdo()
    }
}
