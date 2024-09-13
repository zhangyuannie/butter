use gtk::subclass::prelude::*;
use zbus::zvariant::OwnedObjectPath;

mod imp {
    use std::cell::{OnceCell, RefCell};

    use gtk::{glib, prelude::*, subclass::prelude::*};
    use zbus::zvariant::OwnedObjectPath;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Filesystem)]
    pub struct Filesystem {
        pub path: OnceCell<OwnedObjectPath>,
        #[property(get)]
        pub uuid: RefCell<String>,
        #[property(get)]
        pub label: RefCell<String>,
        #[property(get)]
        pub devices: RefCell<Vec<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Filesystem {
        const NAME: &'static str = "BtrFilesystem";
        type Type = super::Filesystem;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Filesystem {}
}

gtk::glib::wrapper! {
    pub struct Filesystem(ObjectSubclass<imp::Filesystem>);
}

impl Filesystem {
    pub fn new(path: OwnedObjectPath, uuid: String, label: String, devices: Vec<String>) -> Self {
        let ret: Self = gtk::glib::Object::new();
        let imp = ret.imp();
        imp.path.set(path);
        imp.uuid.replace(uuid);
        imp.label.replace(label);
        imp.devices.replace(devices);
        ret
    }

    pub fn display(&self) -> String {
        if self.label().is_empty() {
            self.devices()
                .first()
                .cloned()
                .unwrap_or("(unknown)".to_owned())
        } else {
            self.label()
        }
    }

    pub fn object_path(&self) -> &OwnedObjectPath {
        &self.imp().path.get().unwrap()
    }
}
