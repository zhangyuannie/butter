use zbus::{interface, zvariant::ObjectPath};

use crate::{ZPathBuf, ZUuid};

pub struct Filesystem {
    pub(crate) uuid: ZUuid,
    pub(crate) label: String,
    /// Must be Sorted
    pub(crate) devices: Vec<ZPathBuf>,
    /// Must be Sorted
    pub(crate) mount_points: Vec<ZPathBuf>,
}

impl Filesystem {
    pub(crate) async fn update(
        self,
        server: &zbus::ObjectServer,
        path: ObjectPath<'_>,
    ) -> anyhow::Result<()> {
        let iface_ref = server.interface::<_, Filesystem>(path).await?;
        let mut iface = iface_ref.get_mut().await;

        if iface.uuid != self.uuid {
            iface.uuid = self.uuid;
            iface.uuid_changed(iface_ref.signal_context()).await?;
        }

        if iface.label != self.label {
            iface.label = self.label;
            iface.label_changed(iface_ref.signal_context()).await?;
        }

        if iface.devices != self.devices {
            iface.devices = self.devices;
            iface.devices_changed(iface_ref.signal_context()).await?;
        }

        Ok(())
    }

    pub(crate) async fn create(
        self,
        server: &zbus::ObjectServer,
        path: ObjectPath<'_>,
    ) -> anyhow::Result<()> {
        server.at(path, self).await?;
        Ok(())
    }
}

#[interface(
    name = "org.zhangyuannie.Butter1.Filesystem",
    proxy(gen_blocking = false, default_service = "org.zhangyuannie.Butter1")
)]
impl Filesystem {
    #[zbus(property)]
    fn uuid(&self) -> ZUuid {
        self.uuid
    }

    #[zbus(property)]
    fn label(&self) -> String {
        self.label.clone()
    }

    #[zbus(property)]
    fn devices(&self) -> Vec<ZPathBuf> {
        self.devices.clone()
    }

    #[zbus(property)]
    fn mount_points(&self) -> Vec<ZPathBuf> {
        self.mount_points.clone()
    }
}
