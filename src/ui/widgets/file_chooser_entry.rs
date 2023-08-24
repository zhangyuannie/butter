use gtk::glib;

mod imp {
    use crate::ui::prelude::*;

    use super::*;
    use gtk::{gio, CompositeTemplate};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/file_chooser_entry.ui")]
    pub struct FileChooserEntry {
        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub browse_button: TemplateChild<gtk::Button>,
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
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            self.browse_button
                .connect_clicked(glib::clone!(@weak obj => move |_| {
                    let window = obj.root().unwrap().downcast::<gtk::Window>().unwrap();
                    let file_chooser = gtk::FileDialog::builder().modal(true).build();

                    file_chooser.select_folder(
                        Some(&window),
                        gio::Cancellable::NONE,
                        glib::clone!(@weak obj => move |response| {
                            match response {
                                Ok(file) => {
                                    let path = file.path().unwrap();
                                    obj.set_text(path.to_str().unwrap());
                                }
                                Err(err) => {
                                    if err.matches(gtk::DialogError::Dismissed) || err.matches(gtk::DialogError::Cancelled) {
                                        return;
                                    }
                                    obj.alert(err.message());
                                }
                            }
                        }),
                    );
                }));
        }
    }
    impl WidgetImpl for FileChooserEntry {}
    impl BinImpl for FileChooserEntry {}
    impl EditableImpl for FileChooserEntry {
        fn delegate(&self) -> Option<gtk::Editable> {
            Some(self.entry.get().upcast())
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
        glib::Object::new()
    }
}

impl Default for FileChooserEntry {
    fn default() -> Self {
        Self::new()
    }
}
