use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    use crate::ui::build_ui;

    #[derive(Default)]
    pub struct Application;

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "ButterApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }
    impl ObjectImpl for Application {}
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
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
        ])
        .unwrap()
    }
}
