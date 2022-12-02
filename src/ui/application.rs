use gettext::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::subvolume::{GBtrfsFilesystem, SubvolumeManager};
use crate::{config, ui::show_error_dialog};

use super::widgets::{AppWindow, ScheduleView, SnapshotView};

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use gtk::glib::{once_cell::sync::Lazy, ParamFlags, ParamSpec, ParamSpecObject, Value};
    use std::cell::RefCell;

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

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "subvolume-manager" => {
                    let input_subvolume_manager = value.get().unwrap();
                    self.subvolume_manager.replace(input_subvolume_manager);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "subvolume-manager" => self.subvolume_manager.borrow().to_value(),
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
    pub fn new(manager: SubvolumeManager) -> Self {
        glib::Object::new(&[
            ("application-id", &Some(config::APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            ("subvolume-manager", &Some(manager)),
        ])
    }

    pub fn subvolume_manager(&self) -> SubvolumeManager {
        self.property("subvolume-manager")
    }

    pub fn build_ui(&self) {
        let window = AppWindow::new(self);
        let view_stack = window.view_stack();
        let header_bar = window.header_bar();

        let snapshot_page = view_stack.add(&SnapshotView::new(&self.subvolume_manager()));
        snapshot_page.set_name(Some("snapshot"));
        snapshot_page.set_title(Some(gettext("Snapshot").as_str()));
        snapshot_page.set_icon_name(Some("edit-copy-symbolic"));

        let schedule_page = view_stack.add(&ScheduleView::new(
            &self.subvolume_manager().schedule_repo(),
        ));
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
                glib::closure!(|sv: GBtrfsFilesystem| sv.display()),
            );

            let fs_dropdown = header_bar.fs_dropdown();
            fs_dropdown.set_expression(Some(&exp));
            fs_dropdown.set_model(Some(self.subvolume_manager().filesystems()));

            let app = self.clone();
            fs_dropdown.connect_selected_notify(move |dd| {
                if let Some(fs) = dd.selected_item() {
                    let fs: GBtrfsFilesystem =
                        fs.downcast().expect("Object must be GBtrfsFilesystem");
                    let fs = fs.data().clone();
                    app.subvolume_manager().set_filesystem(fs).unwrap();
                }
            });
        }
        {
            let subvol_mgr = self.subvolume_manager();

            let switch = header_bar.switch();
            switch.set_state(self.subvolume_manager().is_schedule_enabled());
            switch.connect_state_set(glib::clone!(@weak subvol_mgr => @default-return glib::signal::Inhibit(true), move |switch, state| {
                if let Err(error) = subvol_mgr.set_is_schedule_enabled(state) {
                    show_error_dialog(None::<&gtk::Window>,&error.to_string());
                }
                switch.set_state(subvol_mgr.is_schedule_enabled());
                glib::signal::Inhibit(true)
            }));
        }

        window.present();

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(move |_, _| {
            let dialog = gtk::AboutDialog::builder()
                .logo_icon_name(config::APP_ID)
                .copyright("© 2022 Zhangyuan Nie")
                .license_type(gtk::License::Gpl30Only)
                .program_name(config::APP_NAME)
                .authors(vec!["Zhangyuan Nie".into()])
                .transient_for(&window)
                .modal(true)
                .version(config::APP_VERSION)
                .build();

            dialog.show();
        });
        self.add_action(&about_action);
    }
}
