use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

mod imp {
    use crate::requester::daemon;

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/snapshot_creation_window.ui")]
    pub struct SnapshotCreationWindow {
        #[template_child]
        pub create_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SnapshotCreationWindow {
        const NAME: &'static str = "SnapshotCreationWindow";
        type Type = super::SnapshotCreationWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SnapshotCreationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.imp().create_button.connect_clicked(|_| {
                daemon().create_snapshot("/home", "/var/snapshots/test");
            });
        }
    }
    impl WidgetImpl for SnapshotCreationWindow {}
    impl WindowImpl for SnapshotCreationWindow {}
}

glib::wrapper! {
    pub struct SnapshotCreationWindow(ObjectSubclass<imp::SnapshotCreationWindow>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SnapshotCreationWindow {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create SnapshotCreationWindow")
    }
}

impl Default for SnapshotCreationWindow {
    fn default() -> Self {
        Self::new()
    }
}
