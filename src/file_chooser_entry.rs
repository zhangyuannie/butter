use adw;
use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

mod imp {
    use adw::subclass::prelude::BinImpl;

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/file_chooser_entry.ui")]
    pub struct FileChooserEntry {
        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileChooserEntry {
        const NAME: &'static str = "FileChooserEntry";
        type Type = super::FileChooserEntry;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FileChooserEntry {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            // obj.imp().create_button.connect_clicked(|_| {
            //     daemon().create_snapshot("/home", "/var/snapshots/test");
            // });
        }
    }
    impl WidgetImpl for FileChooserEntry {}
    impl BinImpl for FileChooserEntry {}
}

glib::wrapper! {
    pub struct FileChooserEntry(ObjectSubclass<imp::FileChooserEntry>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl FileChooserEntry {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create FileChooserEntry")
    }

    pub fn text(&self) -> glib::GString {
        self.imp().entry.text()
    }
}

impl Default for FileChooserEntry {
    fn default() -> Self {
        Self::new()
    }
}
