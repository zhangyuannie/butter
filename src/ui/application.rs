use gettext::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::filesystem::GFilesystem;
use crate::{config, ui::show_error_dialog};

use super::store::Store;
use super::widgets::{AppWindow, ScheduleView, SnapshotView};

mod imp {
    use adw::subclass::prelude::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecObject, Value};
    use gtk::{glib, prelude::*};
    use once_cell::sync::{Lazy, OnceCell};

    use crate::ui::store::Store;

    #[derive(Default)]
    pub struct Application {
        pub store: OnceCell<Store>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "BtrApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "store",
                    None,
                    None,
                    Store::static_type(),
                    ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "store" => {
                    self.store.set(value.get().unwrap()).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "store" => self.store.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self) {
            self.parent_activate();
            self.instance().build_ui();
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
    pub fn new(store: &Store) -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            ("store", &Some(store)),
        ])
    }

    pub fn store(&self) -> &Store {
        self.imp().store.get().unwrap()
    }

    pub fn build_ui(&self) {
        let window = AppWindow::new(self);
        let view_stack = window.view_stack();
        let header_bar = window.header_bar();

        let snapshot_page = view_stack.add(&SnapshotView::new(self.store()));
        snapshot_page.set_name(Some("snapshot"));
        snapshot_page.set_title(Some(gettext("Snapshot").as_str()));
        snapshot_page.set_icon_name(Some("edit-copy-symbolic"));

        let schedule_page = view_stack.add(&ScheduleView::new(self.store()));
        schedule_page.set_name(Some("schedule"));
        schedule_page.set_title(Some(gettext("Schedule").as_str()));
        schedule_page.set_icon_name(Some("alarm-symbolic"));

        let view_switcher_title = header_bar.view_switcher_title();
        view_switcher_title.set_stack(Some(&view_stack));
        view_switcher_title
            .bind_property("title-visible", &window.switcher_bar(), "reveal")
            .build();

        // filesystem dropdown
        {
            let exp = gtk::ClosureExpression::new::<String>(
                &[] as &[gtk::Expression],
                glib::closure!(|sv: GFilesystem| sv.display()),
            );

            let fs_dropdown = header_bar.fs_dropdown();
            fs_dropdown.set_expression(Some(&exp));
            fs_dropdown.set_model(Some(self.store().filesystems()));

            let app = self.clone();
            fs_dropdown.connect_selected_notify(move |dd| {
                if let Some(fs) = dd.selected_item() {
                    let fs: GFilesystem = fs.downcast().expect("Object must be GBtrfsFilesystem");
                    let fs = fs.data().clone();
                    app.store().set_filesystem(fs).unwrap();
                }
            });
        }
        {
            let store = self.store();

            let switch = header_bar.switch();
            switch.set_state(self.store().is_schedule_enabled());
            switch.connect_state_set(glib::clone!(@weak store => @default-return glib::signal::Inhibit(true), move |switch, state| {
                if let Err(error) = store.set_is_schedule_enabled(state) {
                    show_error_dialog(None::<&gtk::Window>,&error.to_string());
                }
                switch.set_state(store.is_schedule_enabled());
                glib::signal::Inhibit(true)
            }));
        }

        window.present();

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(move |_, _| {
            let about_window = adw::AboutWindow::builder()
                .application_name(config::APP_NAME)
                .application_icon(config::APP_ID)
                .version(config::APP_VERSION)
                .website("https://github.com/zhangyuannie/butter")
                .copyright("© 2022 Zhangyuan Nie")
                .license_type(gtk::License::Gpl30Only)
                .developers(vec!["Zhangyuan Nie".into()])
                .transient_for(&window)
                .modal(true)
                .translator_credits(&gettext("translator-credits"))
                .build();

            about_window.show();
        });
        self.add_action(&about_action);
    }
}
