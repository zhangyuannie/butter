mod btrfs;
mod interface;
mod ioctl;
mod mnt_entry;
mod subvol;

use std::future;

use anyhow::Result;
use log::info;
use zbus_polkit::policykit1;

use crate::interface::Service;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Creating D-Bus connection");
    let conn = zbus::Connection::system().await?;
    let authority = policykit1::AuthorityProxy::new(&conn).await?;

    conn.object_server()
        .at(
            "/org/zhangyuannie/Butter1",
            Service {
                conn: conn.clone(),
                polkit: authority,
            },
        )
        .await?;
    conn.request_name("org.zhangyuannie.Butter1").await?;

    info!("Well-known name requested");
    future::pending::<()>().await;
    Ok(())
}
