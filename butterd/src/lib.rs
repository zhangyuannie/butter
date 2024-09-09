mod filesystem;
mod mnt;
mod storage;
mod zvariant;

use std::collections::HashMap;
use zbus_polkit::policykit1::{AuthorityProxy, CheckAuthorizationFlags, Subject};

pub use filesystem::*;
pub use mnt::*;
pub use storage::*;
pub use zvariant::*;

pub(crate) trait ToFdo<T> {
    fn to_fdo(self) -> zbus::fdo::Result<T>;
}

impl<T> ToFdo<T> for anyhow::Result<T> {
    fn to_fdo(self) -> zbus::fdo::Result<T> {
        self.map_err(|err| zbus::fdo::Error::Failed(format!("{:#}", err)))
    }
}

#[derive(Clone)]
pub(crate) struct Polkit {
    authority: AuthorityProxy<'static>,
}

impl Polkit {
    pub async fn new(conn: &zbus::Connection) -> zbus::Result<Self> {
        let authority = AuthorityProxy::new(conn).await?;
        Ok(Self { authority })
    }

    pub async fn validate(
        &self,
        header: &zbus::message::Header<'_>,
        action_id: &str,
    ) -> zbus::fdo::Result<()> {
        let subject = Subject::new_for_message_header(header)
            .map_err(|_| zbus::fdo::Error::AuthFailed("Failed to create subject".to_owned()))?;
        let auth = self
            .authority
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
            Err(zbus::fdo::Error::AuthFailed(
                "Sender not authorized".to_string(),
            ))
        }
    }
}
