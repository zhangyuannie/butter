use std::future;

use tracing::info;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    tracing_subscriber::fmt().init();
    info!("Creating D-Bus connection");
    let conn = zbus::Connection::system().await?;

    conn.object_server()
        .at(butterd::Storage::PATH, butterd::Storage::new(&conn).await?)
        .await?;

    conn.object_server()
        .interface::<_, butterd::Storage>(butterd::Storage::PATH)
        .await?
        .get_mut()
        .await
        .refresh(&conn.object_server())
        .await?;

    conn.object_server()
        .at(butterd::Storage::PATH, zbus::fdo::ObjectManager)
        .await?;

    info!("Registering well-known name");
    conn.request_name("org.zhangyuannie.Butter1").await?;

    future::pending::<()>().await;
    Ok(())
}
