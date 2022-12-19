use std::future;

use anyhow::Result;
use butter::daemon::Service;
use log::info;
use zbus_polkit::policykit1;

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
