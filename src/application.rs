use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config;
use crate::subvolume::SubvolumeManager;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use gtk::{
        glib::{once_cell::sync::Lazy, ParamFlags, ParamSpec, ParamSpecObject, Value},
        prelude::*,
    };
    use std::cell::RefCell;

    use crate::{subvolume::SubvolumeManager, ui::build_ui};

    #[derive(Default)]
    pub struct Application {
        pub subvolume_manager: RefCell<Option<SubvolumeManager>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "ButterApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "subvolume-manager",
                    "subvolume-manager",
                    "subvolume-manager",
                    SubvolumeManager::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "subvolume-manager" => {
                    let input_subvolume_manager = value.get().unwrap();
                    self.subvolume_manager.replace(input_subvolume_manager);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "subvolume-manager" => self.subvolume_manager.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self, application: &Self::Type) {
            self.parent_activate(application);
            build_ui(application);
        }
    }
    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new(manager: SubvolumeManager) -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            ("subvolume-manager", &Some(manager)),
        ])
        .expect("Failed to create Application")
    }

    pub fn subvolume_manager(&self) -> SubvolumeManager {
        self.property("subvolume-manager")
    }
}
