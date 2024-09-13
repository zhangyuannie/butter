use zbus::{interface, message::Header, object_server::SignalContext, zvariant::ObjectPath};

use crate::{Polkit, RuleConfig, ToFdo};

pub struct Rule {
    pub(crate) name: String,
    pub(crate) is_enabled: bool,
    pub(crate) polkit: Polkit,
}

static ACTION_ID: &str = "org.zhangyuannie.butter.manage-schedule";

impl Rule {
    pub(crate) async fn maybe_set_is_enabled(
        &mut self,
        ctx: &SignalContext<'_>,
        is_enabled: bool,
    ) -> zbus::Result<()> {
        if self.is_enabled != is_enabled {
            self.is_enabled = is_enabled;
            self.is_enabled_changed(ctx).await?;
        }
        Ok(())
    }
    pub(crate) async fn update(
        self,
        server: &zbus::ObjectServer,
        path: &ObjectPath<'_>,
    ) -> anyhow::Result<()> {
        let iface_ref = server.interface::<_, Rule>(path).await?;
        let mut iface = iface_ref.get_mut().await;

        if iface.name != self.name {
            return Err(anyhow::anyhow!("name mismatch"));
        }

        iface
            .maybe_set_is_enabled(iface_ref.signal_context(), self.is_enabled)
            .await?;

        Ok(())
    }

    pub(crate) async fn create(
        self,
        server: &zbus::ObjectServer,
        path: &ObjectPath<'_>,
    ) -> anyhow::Result<()> {
        server.at(path, self).await?;
        Ok(())
    }
}

#[interface(
    name = "org.zhangyuannie.Butter1.Rule",
    proxy(gen_blocking = true, default_service = "org.zhangyuannie.Butter1")
)]
impl Rule {
    #[zbus(property(emits_changed_signal = "const"))]
    fn name(&self) -> String {
        self.name.clone()
    }

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
    ) -> zbus::fdo::Result<()> {
        self.polkit.validate(&header, ACTION_ID).await?;

        let mut cfg = RuleConfig::read(&self.name).to_fdo()?;
        cfg.is_enabled = is_enabled;
        cfg.write(&self.name, false).to_fdo()?;

        self.maybe_set_is_enabled(&ctx, is_enabled).await?;
        Ok(())
    }

    async fn config(
        &mut self,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
    ) -> zbus::fdo::Result<RuleConfig> {
        let config = RuleConfig::read(&self.name).to_fdo()?;

        if self.is_enabled != config.is_enabled {
            self.is_enabled = config.is_enabled;
            self.is_enabled_changed(&ctx).await?;
        }

        Ok(config)
    }

    async fn set_config(
        &mut self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
        config: RuleConfig,
    ) -> zbus::fdo::Result<()> {
        self.polkit.validate(&header, ACTION_ID).await?;

        config.write(&self.name, false).to_fdo()?;

        self.maybe_set_is_enabled(&ctx, config.is_enabled).await?;

        Ok(())
    }
}
