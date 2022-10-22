mod interface;

use anyhow::Result;
use log::info;
use zbus::ConnectionBuilder;

use crate::interface::Service;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Creating D-Bus connection");
    let _ = ConnectionBuilder::system()?
        .name("org.zhangyuannie.Butter1")?
        .serve_at("/org/zhangyuannie/Butter", Service)?
        .build()
        .await?;

    Ok(())
}
