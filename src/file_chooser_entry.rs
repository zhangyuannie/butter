use adw;
use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/file_chooser_entry.ui")]
    pub struct FileChooserEntry {
        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub browse_button: TemplateChild<gtk::Button>,
        pub file_chooser: Rc<RefCell<Option<gtk::FileChooserNative>>>,
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
            self.browse_button
                .connect_clicked(glib::clone!(@weak obj => move |_| {
                    let window = obj.root().unwrap().downcast::<gtk::Window>().unwrap();
                    let file_chooser = gtk::FileChooserNative::builder()
                        .transient_for(&window)
                        .action(gtk::FileChooserAction::SelectFolder)
                        .build();

                    file_chooser.connect_response(glib::clone!(@weak obj => move |chooser, resp_type| {
                        match resp_type {
                            gtk::ResponseType::Accept => {
                                if let Some(file) = chooser.file() {
                                    let path = file.path().unwrap();
                                    obj.set_text(path.to_str().unwrap());
                                }
                            },
                            _ => {},
                        };
                        obj.imp().file_chooser.take();
                    }));

                    file_chooser.show();
                    obj.imp().file_chooser.replace(Some(file_chooser));
                }));
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

    pub fn set_text(&self, text: &str) {
        self.imp().entry.set_text(text);
    }
}

impl Default for FileChooserEntry {
    fn default() -> Self {
        Self::new()
    }
}
