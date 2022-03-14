use crate::application::Application;
use crate::snapshot_view::SnapshotView;
use gtk::prelude::Cast;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, subclass::prelude::*, CompositeTemplate};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub content_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub switcher_bar: TemplateChild<adw::ViewSwitcherBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "ButterWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let new_action = gio::SimpleAction::new("new", None);

            let win = obj.clone();
            new_action.connect_activate(move |_, _| {
                let imp = win.imp();
                let cur_view = imp.view_stack.visible_child_name().unwrap();
                if cur_view == "snapshot" {
                    let view = win.snapshot_view();
                    view.present_creation_window();
                } else {
                    todo!();
                }
            });

            obj.add_action(&new_action);
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow,gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
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
}
