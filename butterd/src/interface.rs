use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::{dbus_interface, zvariant::Type, DBusError, MessageHeader};
use zbus_polkit::policykit1::{self, CheckAuthorizationFlags, Subject};

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

static READ_FILESYSTEM: &str = "org.zhangyuannie.butter.read-filesystem";

#[dbus_interface(name = "org.zhangyuannie.Butter1")]
impl Service<'static> {
    async fn list_filesystems(
        &self,
        #[zbus(header)] hdr: MessageHeader<'_>,
    ) -> Result<Vec<BtrfsFilesystem>, Error> {
        self.check_authorization(&hdr, READ_FILESYSTEM).await?;
        let ret = btrfs::read_all_mounted_btrfs_fs().context("failed to read mounted btrfs fs")?;
        Ok(ret)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Type)]
pub struct BtrfsFilesystem {
    pub label: String,
    pub uuid: Uuid,
    // TODO: PathBuf
    pub devices: Vec<String>,
}
