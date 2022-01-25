use crate::application::Application;
use crate::snapshot_view::SnapshotView;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gtk::{glib, subclass::prelude::*, CompositeTemplate};

    use crate::snapshot_view::SnapshotView;

    #[derive(CompositeTemplate, Default)]
    #[template(file = "../data/resources/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub content_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub switcher_bar: TemplateChild<adw::ViewSwitcherBar>,
        #[template_child]
        pub snapshot_view: TemplateChild<SnapshotView>,
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

    impl ObjectImpl for Window {}
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
        self.imp().snapshot_view.get()
    }

    pub fn switcher_bar(&self) -> adw::ViewSwitcherBar {
        self.imp().switcher_bar.get()
    }
}
