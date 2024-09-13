use std::collections::HashMap;

use anyhow::Context;
use zbus::{
    fdo, interface,
    message::Header,
    object_server::SignalContext,
    zvariant::{ObjectPath, OwnedObjectPath},
};
use zbus_systemd::systemd1;

use crate::{object_path_escape, Polkit, ReadScheduleDir, Rule, RuleConfig, ToFdo};

pub struct Schedule {
    is_enabled: bool,
    rules: HashMap<String, OwnedObjectPath>,
    conn: zbus::Connection,
    polkit: Polkit,
}

static ACTION_ID: &str = "org.zhangyuannie.butter.manage-schedule";
static SNAPSHOT_UNIT: &str = "butter-schedule-snapshot.timer";
static PRUNE_UNIT: &str = "butter-schedule-prune.timer";

impl Schedule {
    pub const PATH: ObjectPath<'static> =
        ObjectPath::from_static_str_unchecked("/org/zhangyuannie/Butter1/Schedule");

    pub async fn new(conn: zbus::Connection) -> zbus::Result<Self> {
        Ok(Self {
            is_enabled: false,
            rules: Default::default(),
            polkit: Polkit::new(&conn).await?,
            conn,
        })
    }

    fn object_path(name: &str) -> OwnedObjectPath {
        // unwrap: name is escaped, the joined path must be valid
        OwnedObjectPath::try_from(format!(
            "{}/{}",
            Self::PATH,
            object_path_escape(name.as_bytes())
        ))
        .unwrap()
    }

    async fn refresh_is_enabled(&mut self, ctx: &SignalContext<'_>) -> zbus::Result<()> {
        let systemd = systemd1::ManagerProxy::new(&self.conn).await?;
        let p = match systemd.get_unit(SNAPSHOT_UNIT.into()).await {
            Ok(p) => p,
            Err(_) => systemd.load_unit(SNAPSHOT_UNIT.into()).await?,
        };

        let unit = systemd1::UnitProxy::new(&self.conn, p).await?;
        let state = unit.active_state().await?;

        let is_enabled = state == "active" || state == "reloading";

        if self.is_enabled != is_enabled {
            self.is_enabled = is_enabled;
            self.is_enabled_changed(ctx).await?;
        }

        Ok(())
    }

    async fn up(
        &mut self,
        server: &zbus::ObjectServer,
        rule: Rule,
    ) -> anyhow::Result<OwnedObjectPath> {
        let path = Self::object_path(&rule.name);
        let name = rule.name.clone();
        rule.create(server, &path).await?;
        self.rules.insert(name, path.clone());
        Ok(path)
    }

    async fn down(&mut self, server: &zbus::ObjectServer, name: &str) -> anyhow::Result<()> {
        let path = Self::object_path(name);
        server.remove::<Rule, _>(path).await?;
        self.rules.remove(name);
        Ok(())
    }
}

#[interface(
    name = "org.zhangyuannie.Butter1.Schedule",
    proxy(
        gen_blocking = true,
        default_service = "org.zhangyuannie.Butter1",
        default_path = "/org/zhangyuannie/Butter1/Schedule",
    )
)]
impl Schedule {
    #[zbus(property)]
    fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    // can not use property: https://github.com/dbus2/zbus/issues/218
    async fn set_is_enabled(
        &mut self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(signal_context)] ctx: SignalContext<'_>,

        is_enabled: bool,
    ) -> fdo::Result<()> {
        self.polkit.validate(&header, ACTION_ID).await?;

        let systemd = systemd1::ManagerProxy::new(&self.conn).await?;

        let units = vec![SNAPSHOT_UNIT.to_owned(), PRUNE_UNIT.to_owned()];

        if is_enabled {
            systemd.enable_unit_files(units, false, true).await?;
            futures::future::try_join(
                systemd.start_unit(SNAPSHOT_UNIT.into(), "replace".into()),
                systemd.start_unit(PRUNE_UNIT.into(), "replace".into()),
            )
            .await?;
        } else {
            systemd.disable_unit_files(units, false).await?;
            futures::future::try_join(
                systemd.stop_unit(SNAPSHOT_UNIT.into(), "replace".into()),
                systemd.stop_unit(PRUNE_UNIT.into(), "replace".into()),
            )
            .await?;
        }

        self.refresh_is_enabled(&ctx).await?;

        Ok(())
    }

    pub async fn refresh(
        &mut self,
        #[zbus(object_server)] server: &zbus::ObjectServer,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
    ) -> fdo::Result<()> {
        let dir = ReadScheduleDir::new()
            .context("Failed to read config directroy")
            .to_fdo()?;

        let configs: HashMap<_, _> = dir.flatten().collect();

        let to_remove: Vec<_> = self
            .rules
            .iter()
            .filter_map(|r| configs.get(r.0).map(|_| (r.0.clone(), r.1.clone())))
            .collect();

        let remove_futs = to_remove
            .iter()
            .map(|(_, path)| server.remove::<Rule, _>(path));
        futures::future::join_all(remove_futs).await;

        for (name, _) in to_remove {
            self.rules.remove(&name);
        }

        for (name, config) in configs {
            let rule = Rule {
                name: name.clone(),
                is_enabled: config.is_enabled,
                polkit: self.polkit.clone(),
            };
            if let Some(path) = self.rules.get(&name) {
                rule.update(server, path).await.to_fdo()?;
            } else {
                let path = Self::object_path(&name);
                rule.create(server, &path).await.to_fdo()?;
                self.rules.insert(name, path);
            }
        }

        self.refresh_is_enabled(&ctx).await?;
        Ok(())
    }

    async fn create_rule(
        &mut self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(object_server)] server: &zbus::ObjectServer,
        name: String,
        config: RuleConfig,
    ) -> fdo::Result<OwnedObjectPath> {
        self.polkit.validate(&header, ACTION_ID).await?;

        config
            .write(&name, true)
            .context("failed to write")
            .to_fdo()?;

        let rule = Rule {
            name,
            is_enabled: config.is_enabled,
            polkit: self.polkit.clone(),
        };

        self.up(server, rule).await.to_fdo()
    }

    async fn remove_rule(
        &mut self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(object_server)] server: &zbus::ObjectServer,
        name: String,
    ) -> zbus::fdo::Result<()> {
        self.polkit.validate(&header, ACTION_ID).await?;
        tokio::fs::remove_file(RuleConfig::path(&name))
            .await
            .to_fdo()?;

        self.down(server, &name).await.to_fdo()
    }

    async fn move_rule(
        &mut self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(object_server)] server: &zbus::ObjectServer,
        prev: String,
        next: String,
    ) -> zbus::fdo::Result<OwnedObjectPath> {
        self.polkit.validate(&header, ACTION_ID).await?;
        let dst = RuleConfig::path(&next);
        if dst.exists() {
            // best efforts
            return Err(fdo::Error::InvalidArgs("Target already exists".to_owned()));
        }

        std::fs::rename(RuleConfig::path(&prev), dst).to_fdo()?;
        self.down(server, &next).await.to_fdo()?;

        let config = RuleConfig::read(&next).to_fdo()?;

        let rule = Rule {
            name: next,
            is_enabled: config.is_enabled,
            polkit: self.polkit.clone(),
        };
        self.up(server, rule).await.to_fdo()
    }

    async fn get_rule(
        &mut self,
        name: String,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) -> zbus::fdo::Result<OwnedObjectPath> {
        if let Some(p) = self.rules.get(&name) {
            return Ok(p.clone());
        }

        // try loading it
        let config = RuleConfig::read(&name).to_fdo()?;

        let rule = Rule {
            name,
            is_enabled: config.is_enabled,
            polkit: self.polkit.clone(),
        };

        self.up(server, rule).await.to_fdo()
    }
}
