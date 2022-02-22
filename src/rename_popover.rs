use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

mod imp {
    use gtk::glib::{once_cell::sync::Lazy, subclass::Signal};
    use std::cell::Cell;

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(file = "../data/resources/ui/rename_popover.ui")]
    pub struct RenamePopover {
        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub rename_button: TemplateChild<gtk::Button>,
        pub idx: Cell<u32>,
    }

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

    impl ObjectImpl for RenamePopover {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            let btn = self.rename_button.get();
            let entry = self.entry.get();
            let popover = obj.clone();
            btn.connect_clicked(move |_| {
                let new_name = entry.text();
                popover.emit_by_name::<()>("clicked", &[&new_name]);
            });
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "clicked",
                    &[String::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }
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

    pub fn set_target(&self, idx: u32) {
        self.imp().idx.set(idx);
    }

    pub fn target(&self) -> u32 {
        self.imp().idx.get()
    }
}

impl Default for RenamePopover {
    fn default() -> Self {
        Self::new()
    }
}
