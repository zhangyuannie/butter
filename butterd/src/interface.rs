use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::try_join;
use uuid::Uuid;
use zbus::{dbus_interface, zvariant::Type, Connection, DBusError, MessageHeader};
use zbus_polkit::policykit1::{self, CheckAuthorizationFlags, Subject};
use zbus_systemd::systemd1;

use crate::btrfs;

#[derive(DBusError, Debug)]
#[dbus_error(prefix = "org.zhangyuannie.Butter1")]
enum Error {
    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),
    Failed(String),
    AuthFailed(String),
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
static SCHEDULE_AID: &str = "org.zhangyuannie.butter.schedule";

#[dbus_interface(name = "org.zhangyuannie.Butter1")]
impl Service<'static> {
    async fn list_filesystems(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
    ) -> Result<Vec<BtrfsFilesystem>, Error> {
        self.check_authorization(&hdr, FILESYSTEM_AID).await?;
        let ret = btrfs::read_all_mounted_btrfs_fs().context("failed to read mounted btrfs fs")?;
        Ok(ret)
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
        let p = systemd
            .get_unit("butter-schedule-snapshot.timer".to_string())
            .await?;

        let unit = systemd1::UnitProxy::new(&self.conn, p).await?;
        let state = unit.active_state().await?;
        if state == "active" || state == "reloading" {
            Ok("active")
        } else {
            Ok("inactive")
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Type)]
pub struct BtrfsFilesystem {
    pub label: String,
    pub uuid: Uuid,
    // TODO: PathBuf
    pub devices: Vec<String>,
}
