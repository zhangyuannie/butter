use std::{collections::HashMap, fs, io};

use anyhow::Context;
use libblkid_rs::{evaluate_spec, BlkidCache};
use uuid::Uuid;
use zbus::{
    fdo, interface,
    message::Header,
    zvariant::{ObjectPath, OwnedObjectPath},
};

use crate::{create_snapshot, Filesystem, MntEntries, Polkit, ToFdo, ZPathBuf};

pub struct Storage {
    filesystems: HashMap<Uuid, OwnedObjectPath>,
    polkit: Polkit,
}

static ACTION_ID: &str = "org.zhangyuannie.butter.manage-subvolume";

impl Storage {
    pub const PATH: ObjectPath<'static> =
        ObjectPath::from_static_str_unchecked("/org/zhangyuannie/Butter1/Storage");

    pub async fn new(polkit: Polkit) -> zbus::Result<Self> {
        Ok(Self {
            filesystems: Default::default(),
            polkit,
        })
    }

    fn subvol_id_from_mnt_options(options: &str) -> Option<u64> {
        for seg in options.split(',') {
            if let Some(id) = seg.strip_prefix("subvolid=") {
                return id.parse::<u64>().ok();
            }
        }
        None
    }

    fn probe_btrfs_devices() -> anyhow::Result<HashMap<Uuid, Filesystem>> {
        let mut ret = HashMap::new();

        let mut cache = BlkidCache::get_cache(None)?;
        cache.probe_all()?;

        let mut uuid_by_devname = HashMap::new();

        for dev in cache.iter().search("TYPE", "btrfs")? {
            if let Some(dev) = cache.verify(dev) {
                let devname = dev.devname()?;
                let uuid = cache.get_tag_value("UUID", &devname)?;
                let uuid = Uuid::try_parse(&uuid)?;
                let label = cache.get_tag_value("LABEL", &devname)?;
                let vref = ret.entry(uuid).or_insert(Filesystem {
                    uuid: uuid.into(),
                    label,
                    devices: Vec::new(),
                    mount_points_by_subvol_id: Default::default(),
                });

                uuid_by_devname.insert(devname.clone(), uuid);

                vref.devices.push(devname.into())
            }
        }

        let f = fs::File::open("/proc/self/mounts")?;
        let entries = MntEntries::new(io::BufReader::new(f));
        for entry in entries.flatten() {
            if entry.fs_type != "btrfs" {
                continue;
            }
            let mnt_path = entry.target.context("failed to read target")?;
            let devname = evaluate_spec(&entry.spec, Some(&mut cache))?;
            if let Some(uuid) = uuid_by_devname.get(&devname) {
                let subvol_id = Self::subvol_id_from_mnt_options(&entry.options)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidData))?;
                // unwrap: if uuid in uuid_by_devname, then it must be in ret
                ret.get_mut(uuid)
                    .unwrap()
                    .mount_points_by_subvol_id
                    .entry(subvol_id)
                    .or_default()
                    .push(mnt_path.into());
            }
        }

        for fs in ret.values_mut() {
            fs.devices.sort_unstable();
        }

        Ok(ret)
    }

    async fn refresh_impl(&mut self, server: &zbus::ObjectServer) -> anyhow::Result<()> {
        let next_filesystems = Self::probe_btrfs_devices()?;

        let mut to_remove = Vec::new();
        for (uuid, path) in &self.filesystems {
            if !next_filesystems.contains_key(uuid) {
                to_remove.push((*uuid, path.clone()));
            }
        }
        let remove_futs = to_remove
            .iter()
            .map(|(_, path)| server.remove::<Filesystem, _>(path));
        futures::future::join_all(remove_futs).await;

        for (uuid, _) in to_remove {
            self.filesystems.remove(&uuid);
        }

        for (uuid, fs) in next_filesystems {
            if let Some(path) = self.filesystems.get(&uuid) {
                fs.update(server, path).await?;
            } else {
                let path = OwnedObjectPath::try_from(format!("{}/{}", Self::PATH, uuid.simple()))?;
                fs.create(server, &path).await?;
                self.filesystems.insert(uuid, path);
            }
        }

        Ok(())
    }
}

#[interface(
    name = "org.zhangyuannie.Butter1.Storage",
    proxy(
        gen_blocking = true,
        default_service = "org.zhangyuannie.Butter1",
        default_path = "/org/zhangyuannie/Butter1/Storage",
    )
)]
impl Storage {
    pub async fn refresh(
        &mut self,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) -> fdo::Result<()> {
        self.refresh_impl(server).await.to_fdo()
    }

    pub async fn remove_subvolumes(
        &self,
        #[zbus(header)] header: Header<'_>,
        paths: Vec<ZPathBuf>,
    ) -> fdo::Result<()> {
        self.polkit.validate(&header, ACTION_ID).await?;

        for p in paths {
            if p.as_path().is_relative() {
                return Err(fdo::Error::InvalidArgs("Path must be absolute".to_owned()));
            }
            libbtrfsutil::DeleteSubvolumeOptions::new()
                .recursive(true)
                .delete(p.as_path())
                .context("Failed to delete subvolume")
                .to_fdo()?;
        }

        Ok(())
    }

    pub async fn move_subvolume(
        &self,
        #[zbus(header)] header: Header<'_>,
        src_path: ZPathBuf,
        dst_path: ZPathBuf,
    ) -> fdo::Result<()> {
        self.polkit.validate(&header, ACTION_ID).await?;
        if src_path.as_path().is_relative() || dst_path.as_path().is_relative() {
            return Err(fdo::Error::InvalidArgs("Path must be absolute".to_owned()));
        }

        if dst_path.as_path().exists() {
            // best efforts
            return Err(fdo::Error::InvalidArgs("Target already exists".to_owned()));
        }

        fs::rename(src_path.as_path(), dst_path.as_path())
            .context("Failed to move subvolume")
            .to_fdo()?;

        Ok(())
    }

    pub async fn create_snapshot(
        &self,
        #[zbus(header)] header: Header<'_>,
        src_path: ZPathBuf,
        dst_path: ZPathBuf,
        readonly: bool,
    ) -> fdo::Result<()> {
        self.polkit.validate(&header, ACTION_ID).await?;
        if src_path.as_path().is_relative() || dst_path.as_path().is_relative() {
            return Err(fdo::Error::InvalidArgs("Path must be absolute".to_owned()));
        }

        create_snapshot(src_path.as_path(), dst_path.as_path(), readonly)
            .context("Failed to create snapshot")
            .to_fdo()?;

        Ok(())
    }
}
