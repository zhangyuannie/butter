use tracing::info;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    tracing_subscriber::fmt().init();
    info!("Creating D-Bus connection");
    let conn = zbus::Connection::system().await?;

    let polkit = butterd::Polkit::new(&conn).await?;

    conn.object_server()
        .at(
            butterd::Storage::PATH,
            butterd::Storage::new(polkit.clone()).await?,
        )
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

    conn.object_server()
        .at(
            butterd::Schedule::PATH,
            butterd::Schedule::new(conn.clone()).await?,
        )
        .await?;

    let schedule = conn
        .object_server()
        .interface::<_, butterd::Schedule>(butterd::Schedule::PATH)
        .await?;

    schedule
        .get_mut()
        .await
        .refresh(&conn.object_server(), schedule.signal_context().clone())
        .await?;

    conn.object_server()
        .at(butterd::Schedule::PATH, zbus::fdo::ObjectManager)
        .await?;

    info!("Registering well-known name");
    conn.request_name("org.zhangyuannie.Butter1").await?;

    loop {
        let listener = conn.monitor_activity();
        let d_15min = std::time::Duration::from_secs(15 * 60);
        if tokio::time::timeout(d_15min, listener).await.is_err() {
            info!("Exiting due to inactivity");
            break;
        }
    }
    Ok(())
}
