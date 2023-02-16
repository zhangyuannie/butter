use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

mod imp {
    use gtk::glib::{once_cell::sync::Lazy, subclass::Signal};

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/snapshot_rename_popover.ui")]
    pub struct SnapshotRenamePopover {
        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub rename_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SnapshotRenamePopover {
        const NAME: &'static str = "SnapshotRenamePopover";
        type Type = super::SnapshotRenamePopover;
        type ParentType = gtk::Popover;
        type Interfaces = (gtk::Editable,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SnapshotRenamePopover {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("clicked").build()]);
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for SnapshotRenamePopover {}
    impl PopoverImpl for SnapshotRenamePopover {}
    impl EditableImpl for SnapshotRenamePopover {
        fn delegate(&self) -> Option<gtk::Editable> {
            Some(self.entry.get().upcast())
        }
    }
}

glib::wrapper! {
    pub struct SnapshotRenamePopover(ObjectSubclass<imp::SnapshotRenamePopover>)
        @extends gtk::Popover, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Native, gtk::ShortcutManager,
                    gtk::Editable;
}

#[gtk::template_callbacks]
impl SnapshotRenamePopover {
    pub fn new() -> Self {
        Object::new()
    }

    #[template_callback]
    fn on_rename_button_clicked(&self) {
        self.emit_by_name::<()>("clicked", &[]);
    }

    pub fn connect_clicked<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "clicked",
            true,
            glib::closure_local!(move |obj: Self| {
                f(&obj);
            }),
        )
    }
}

impl Default for SnapshotRenamePopover {
    fn default() -> Self {
        Self::new()
    }
}
