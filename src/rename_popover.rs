use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(file = "../data/resources/ui/rename_popover.ui")]
    pub struct RenamePopover {}

    #[glib::object_subclass]
    impl ObjectSubclass for RenamePopover {
        const NAME: &'static str = "RenamePopover";
        type Type = super::RenamePopover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RenamePopover {}
    impl WidgetImpl for RenamePopover {}
    impl PopoverImpl for RenamePopover {}
}

glib::wrapper! {
    pub struct RenamePopover(ObjectSubclass<imp::RenamePopover>)
        @extends gtk::Popover, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Native, gtk::ShortcutManager;
}

impl RenamePopover {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create RenamePopover")
    }
}

impl Default for RenamePopover {
    fn default() -> Self {
        Self::new()
    }
}
