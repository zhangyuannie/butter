use butterd::BtrfsFilesystem;
use glib::Object;
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    use glib::once_cell::sync::OnceCell;

    use gtk::glib::{self, once_cell::sync::Lazy, ParamFlags, ParamSpec, Value};

    #[derive(Default)]
    pub struct GBtrfsFilesystem {
        pub data: OnceCell<BtrfsFilesystem>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GBtrfsFilesystem {
        const NAME: &'static str = "BtrfsFilesystem";
        type Type = super::GBtrfsFilesystem;
    }

    impl ObjectImpl for GBtrfsFilesystem {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::new(
                    "label",
                    "Label",
                    "Label of a Btrfs filesystem",
                    None,
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "label" => self.instance().label().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct GBtrfsFilesystem(ObjectSubclass<imp::GBtrfsFilesystem>);
}

impl GBtrfsFilesystem {
    pub fn new(fs: BtrfsFilesystem) -> Self {
        let obj: Self = Object::new(&[]);
        obj.imp().data.set(fs).unwrap();
        obj
    }

    pub fn data(&self) -> &BtrfsFilesystem {
        self.imp().data.get().unwrap()
    }

    pub fn label(&self) -> &str {
        self.data().label.as_str()
    }

    pub fn display(&self) -> String {
        if self.label().is_empty() {
            format!("\"{}\"", self.data().devices[0].display())
        } else {
            self.label().to_string()
        }
    }
}

impl From<GBtrfsFilesystem> for BtrfsFilesystem {
    fn from(fs: GBtrfsFilesystem) -> Self {
        fs.data().clone()
    }
}
