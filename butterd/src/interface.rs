use zbus::dbus_interface;

pub struct Service;

#[dbus_interface(name = "org.zhangyuannie.Butter1")]
impl Service {}
