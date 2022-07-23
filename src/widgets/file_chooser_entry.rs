use gtk::{glib, prelude::*};

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use gtk::{subclass::prelude::*, CompositeTemplate};
    use std::cell::RefCell;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/file_chooser_entry.ui")]
    pub struct FileChooserEntry {
        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub browse_button: TemplateChild<gtk::Button>,
        pub file_chooser: RefCell<Option<gtk::FileChooserNative>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileChooserEntry {
        const NAME: &'static str = "FileChooserEntry";
        type Type = super::FileChooserEntry;
        type ParentType = adw::Bin;
        type Interfaces = (gtk::Editable,);

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

                    obj.imp().file_chooser.replace(Some(file_chooser));
                    let file_chooser = obj.imp().file_chooser.borrow().clone().unwrap();

                    file_chooser.connect_response(glib::clone!(@weak obj => move |chooser, resp_type| {
                        if resp_type == gtk::ResponseType::Accept {
                            if let Some(file) = chooser.file() {
                                let path = file.path().unwrap();
                                obj.set_text(path.to_str().unwrap());
                            }
                        };
                        obj.imp().file_chooser.take();
                    }));

                    file_chooser.show();
                }));
        }
    }
    impl WidgetImpl for FileChooserEntry {}
    impl BinImpl for FileChooserEntry {}
    impl EditableImpl for FileChooserEntry {
        fn delegate(&self, editable: &Self::Type) -> Option<gtk::Editable> {
            Some(editable.imp().entry.get().upcast())
        }
    }
}

glib::wrapper! {
    pub struct FileChooserEntry(ObjectSubclass<imp::FileChooserEntry>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Editable;
}

impl FileChooserEntry {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create FileChooserEntry")
    }
}

impl Default for FileChooserEntry {
    fn default() -> Self {
        Self::new()
    }
}
