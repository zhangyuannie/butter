mod subvolume_manager;

use butter::daemon::interface;
pub use subvolume_manager::SubvolumeManager;

use std::{cell::Ref, path::Path, time::SystemTime};

use glib::Object;
use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::Deserialize;

mod imp {
    use super::*;
    use glib::once_cell::sync::OnceCell;
    use std::{cell::RefCell, rc::Rc};

    use gtk::glib::{self, once_cell::sync::Lazy, ParamFlags, ParamSpec, ParamSpecString, Value};

    #[derive(Default)]
    pub struct GSubvolume {
        pub data: OnceCell<interface::Subvolume>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GSubvolume {
        const NAME: &'static str = "Subvolume";
        type Type = super::GSubvolume;
    }

    impl ObjectImpl for GSubvolume {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("name", "name", "name", None, ParamFlags::READABLE),
                    ParamSpecString::new("path", "path", "path", None, ParamFlags::READWRITE),
                    ParamSpecString::new(
                        "parent-path",
                        "parent-path",
                        "parent-path",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "creation-time",
                        "creation-time",
                        "creation-time",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "absolute-path",
                        "absolute-path",
                        "absolute-path",
                        None,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => obj.name().to_value(),
                "absolute-path" | "path" => obj.path().to_str().to_value(),
                "parent-path" => "x".to_value(),
                "creation-time" => obj.g_created().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct GSubvolume(ObjectSubclass<imp::GSubvolume>);
}

impl GSubvolume {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create Subvolume")
    }

    pub fn data(&self) -> &interface::Subvolume {
        self.imp().data.get().unwrap()
    }

    pub fn name(&self) -> &str {
        self.data().path.file_name().unwrap().to_str().unwrap()
    }

    pub fn path(&self) -> &Path {
        self.data().path.as_path()
    }

    pub fn is_snapshot(&self) -> bool {
        self.data().snapshot_source_uuid.is_some()
    }

    pub fn set_data(&self, data: interface::Subvolume) {
        self.imp().data.set(data).unwrap()
    }

    pub fn g_created(&self) -> glib::DateTime {
        let created = self
            .data()
            .created
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        glib::DateTime::from_unix_local(created.as_secs() as i64).unwrap()
    }
}

impl From<interface::Subvolume> for GSubvolume {
    fn from(item: interface::Subvolume) -> Self {
        let ret = GSubvolume::new();
        ret.set_data(item);
        ret
    }
}
