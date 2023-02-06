use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Context;
use tokio::try_join;
use zbus::{dbus_interface, zvariant::Optional, Connection, DBusError, MessageHeader};
use zbus_polkit::policykit1::{self, CheckAuthorizationFlags, Subject};
use zbus_systemd::systemd1;

use crate::{
    filesystem::{Filesystem, SubvolumeExt},
    rule::{ReadRuleDir, Rule, RuleJson},
    subvolume::Subvolume,
    write_as_json,
};

use super::btrfs;

#[derive(DBusError, Debug)]
#[dbus_error(prefix = "org.zhangyuannie.Butter1")]
enum Error {
    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),
    Failed(String),
    AuthFailed(String),
    ClientError(String),
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Failed(e.to_string())
    }
}

pub struct Service<'c> {
    pub conn: Connection,
    pub polkit: policykit1::AuthorityProxy<'c>,
}

impl<'c> Service<'c> {
    async fn check_authorization(
        &self,
        hdr: &MessageHeader<'_>,
        action_id: &str,
    ) -> Result<(), Error> {
        let subject = Subject::new_for_message_header(hdr).map_err(|_| {
            Error::AuthFailed("failed to create subject from message header".to_string())
        })?;
        let auth = self
            .polkit
            .check_authorization(
                &subject,
                action_id,
                &HashMap::new(),
                CheckAuthorizationFlags::AllowUserInteraction.into(),
                "",
            )
            .await?;
        if auth.is_authorized {
            Ok(())
        } else {
            Err(Error::AuthFailed("subject not authorized".to_string()))
        }
    }
}

static FILESYSTEM_AID: &str = "org.zhangyuannie.butter.filesystem";
static SCHEDULE_AID: &str = "org.zhangyuannie.butter.manage-schedule";
static SUBVOLUME_AID: &str = "org.zhangyuannie.butter.manage-subvolume";

#[dbus_interface(name = "org.zhangyuannie.Butter1")]
impl Service<'static> {
    async fn list_filesystems(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
    ) -> Result<Vec<Filesystem>, Error> {
        self.check_authorization(&hdr, FILESYSTEM_AID).await?;
        let ret = btrfs::read_all_mounted_btrfs_fs().context("failed to read mounted btrfs fs")?;
        Ok(ret)
    }

    async fn list_subvolumes(&self, fs: Filesystem) -> Result<Vec<Subvolume>, Error> {
        Ok(fs.subvolumes()?)
    }

    async fn move_subvolume(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        src_mnt: PathBuf,
        dst_mnt: PathBuf,
    ) -> Result<(), Error> {
        self.check_authorization(&hdr, SUBVOLUME_AID).await?;
        if src_mnt.is_relative() || dst_mnt.is_relative() {
            return Err(Error::ClientError("path must be absolute".to_string()));
        }
        fs::rename(src_mnt, dst_mnt).context("failed to move subvolume")?;
        Ok(())
    }

    async fn delete_subvolumes(
        &mut self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        mnts: Vec<PathBuf>,
    ) -> Result<(), Error> {
        self.check_authorization(&hdr, SUBVOLUME_AID).await?;
        for mnt in mnts {
            if mnt.is_relative() {
                return Err(Error::ClientError("path must be absolute".to_string()));
            }
            libbtrfsutil::delete_subvolume(mnt, libbtrfsutil::DeleteSubvolumeFlags::RECURSIVE)
                .context("failed to delete subvolume")?;
        }
        Ok(())
    }

    async fn create_snapshot(
        &mut self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        src_mnt: PathBuf,
        dst_mnt: PathBuf,
        readonly: bool,
    ) -> Result<Subvolume, Error> {
        self.check_authorization(&hdr, SUBVOLUME_AID).await?;
        if src_mnt.is_relative() || dst_mnt.is_relative() {
            return Err(Error::ClientError("path must be absolute".to_string()));
        }
        if let Some(dest_parent) = dst_mnt.parent() {
            fs::create_dir_all(dest_parent).context("failed to create target parent")?;
        }

        btrfs::create_butter_snapshot(&src_mnt, &dst_mnt, readonly)
            .context("failed to create snapshot")?;

        let info = libbtrfsutil::subvolume_info(&dst_mnt).context("failed to get snapshot info")?;

        let subvol_path =
            libbtrfsutil::subvolume_path(&dst_mnt).context("failed to get subvolume path")?;

        Ok(Subvolume {
            subvol_path,
            mount_path: Optional::from(Some(dst_mnt)),
            is_mountpoint: false,
            uuid: info.uuid(),
            id: info.id(),
            created_unix_secs: info.otime(),
            snapshot_source_uuid: Optional::from(info.parent_uuid()),
        })
    }

    async fn enable_schedule(&self, #[zbus(header)] hdr: MessageHeader<'_>) -> Result<(), Error> {
        self.check_authorization(&hdr, SCHEDULE_AID).await?;
        let systemd = systemd1::ManagerProxy::new(&self.conn).await?;

        systemd
            .enable_unit_files(
                vec![
                    "butter-schedule-snapshot.timer".to_string(),
                    "butter-schedule-prune.timer".to_string(),
                ],
                false,
                true,
            )
            .await?;

        try_join!(
            systemd.start_unit(
                "butter-schedule-snapshot.timer".to_string(),
                "replace".to_string(),
            ),
            systemd.start_unit(
                "butter-schedule-prune.timer".to_string(),
                "replace".to_string(),
            )
        )?;

        Ok(())
    }

    async fn disable_schedule(&self, #[zbus(header)] hdr: MessageHeader<'_>) -> Result<(), Error> {
        self.check_authorization(&hdr, SCHEDULE_AID).await?;
        let systemd = systemd1::ManagerProxy::new(&self.conn).await?;

        systemd
            .disable_unit_files(
                vec![
                    "butter-schedule-snapshot.timer".to_string(),
                    "butter-schedule-prune.timer".to_string(),
                ],
                false,
            )
            .await?;

        try_join!(
            systemd.stop_unit(
                "butter-schedule-snapshot.timer".to_string(),
                "replace".to_string(),
            ),
            systemd.stop_unit(
                "butter-schedule-prune.timer".to_string(),
                "replace".to_string(),
            )
        )?;

        Ok(())
    }

    async fn schedule_state(&self) -> Result<&str, Error> {
        let systemd = systemd1::ManagerProxy::new(&self.conn).await?;

        let p = match systemd
            .get_unit("butter-schedule-snapshot.timer".to_string())
            .await
        {
            Ok(p) => p,
            Err(_) => {
                systemd
                    .load_unit("butter-schedule-snapshot.timer".to_string())
                    .await?
            }
        };

        let unit = systemd1::UnitProxy::new(&self.conn, p).await?;
        let state = unit.active_state().await?;
        if state == "active" || state == "reloading" {
            Ok("active")
        } else {
            Ok("inactive")
        }
    }

    async fn list_rules(&self) -> Result<Vec<Rule>, Error> {
        Ok(ReadRuleDir::new()
            .context("Failed to read config directory")?
            .map_while(|e| e.ok())
            .collect())
    }

    async fn update_rule(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        prev: Rule,
        next: Rule,
    ) -> Result<(), Error> {
        self.check_authorization(&hdr, SCHEDULE_AID).await?;
        let next = RuleJson::from(next);
        write_as_json(&prev.path, &next).context("failed to write")?;
        if prev.path != next.path {
            fs::rename(prev.path, next.path).context("failed to rename")?;
        }
        Ok(())
    }

    async fn delete_rule(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        rule: Rule,
    ) -> Result<(), Error> {
        self.check_authorization(&hdr, SCHEDULE_AID).await?;
        Ok(fs::remove_file(&rule.path).context("failed to remove")?)
    }

    async fn create_rule(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
        rule: Rule,
    ) -> Result<(), Error> {
        self.check_authorization(&hdr, SCHEDULE_AID).await?;
        let rule = RuleJson::from(rule);
        write_as_json(&rule.path, &rule).context("failed to write")?;
        Ok(())
    }
}
