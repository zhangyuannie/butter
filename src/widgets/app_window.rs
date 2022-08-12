use crate::application::Application;
use crate::subvolume::SubvolumeManager;
use crate::widgets::{AppHeaderBar, SnapshotView};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, subclass::prelude::*, CompositeTemplate};

    use crate::{config::APP_ID, widgets::AppHeaderBar};

    #[derive(CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/app_window.ui")]
    pub struct AppWindow {
        #[template_child]
        pub content_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub switcher_bar: TemplateChild<adw::ViewSwitcherBar>,
        #[template_child]
        pub header_bar: TemplateChild<AppHeaderBar>,
        pub settings: gio::Settings,
    }

    impl Default for AppWindow {
        fn default() -> Self {
            Self {
                content_box: TemplateChild::default(),
                view_stack: TemplateChild::default(),
                switcher_bar: TemplateChild::default(),
                header_bar: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppWindow {
        const NAME: &'static str = "AppWindow";
        type Type = super::AppWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AppWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.load_window_state();

            let new_action = gio::SimpleAction::new("new", None);

            let win = obj.clone();
            new_action.connect_activate(move |_, _| {
                let imp = win.imp();
                let cur_view = imp.view_stack.visible_child_name().unwrap();
                if cur_view == "snapshot" {
                    let view = win.snapshot_view();
                    view.present_creation_window();
                }
            });

            obj.add_action(&new_action);

            let header_bar = self.header_bar.get();
            self.view_stack.connect_visible_child_name_notify(
                glib::clone!(@weak header_bar => move |vs| {
                    if let Some(view) = vs.visible_child_name() {
                        match view.as_str() {
                            "snapshot" => {
                                header_bar.set_property("title-start", "add");
                                header_bar.set_property("title-end", "fs");
                            }
                            "schedule" => {
                                header_bar.set_property("title-start", "none");
                                header_bar.set_property("title-end", "switch");
                            }
                            _ => unimplemented!(),
                        }
                    }
                }),
            );
        }
    }
    impl WidgetImpl for AppWindow {}
    impl WindowImpl for AppWindow {
        fn close_request(&self, window: &Self::Type) -> gtk::Inhibit {
            if let Err(err) = window.save_window_state() {
                println!("Failed to save window state, {}", &err);
            }
            self.parent_close_request(window)
        }
    }
    impl ApplicationWindowImpl for AppWindow {}
    impl AdwApplicationWindowImpl for AppWindow {}
}

glib::wrapper! {
    pub struct AppWindow(ObjectSubclass<imp::AppWindow>)
        @extends adw::ApplicationWindow,gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl AppWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).unwrap()
    }

    pub fn content_box(&self) -> gtk::Box {
        self.imp().content_box.get()
    }

    pub fn view_stack(&self) -> adw::ViewStack {
        self.imp().view_stack.get()
    }

    pub fn snapshot_view(&self) -> SnapshotView {
        self.view_stack()
            .child_by_name("snapshot")
            .unwrap()
            .downcast()
            .unwrap()
    }

    pub fn switcher_bar(&self) -> adw::ViewSwitcherBar {
        self.imp().switcher_bar.get()
    }

    pub fn header_bar(&self) -> AppHeaderBar {
        self.imp().header_bar.get()
    }

    pub fn subvolume_manager(&self) -> SubvolumeManager {
        self.application()
            .unwrap()
            .downcast_ref::<Application>()
            .unwrap()
            .subvolume_manager()
    }

    fn save_window_state(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let size = self.default_size();
        imp.settings.set("window-size", &size)?;

        imp.settings
            .set_boolean("window-is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_state(&self) {
        let imp = self.imp();

        let (width, height) = imp.settings.get("window-size");
        self.set_default_size(width, height);

        let is_maximized = imp.settings.boolean("window-is-maximized");

        if is_maximized {
            self.maximize();
        }
    }
}
