use butter::daemon::interface;

use glib::Object;
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    use glib::once_cell::sync::OnceCell;

    use gtk::glib::{self, once_cell::sync::Lazy, ParamFlags, ParamSpec, Value};

    #[derive(Default)]
    pub struct GBtrfsFilesystem {
        pub data: OnceCell<interface::BtrfsFilesystem>,
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

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "label" => obj.label().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct GBtrfsFilesystem(ObjectSubclass<imp::GBtrfsFilesystem>);
}

impl GBtrfsFilesystem {
    pub fn new(fs: interface::BtrfsFilesystem) -> Self {
        let obj: Self = Object::new(&[]).expect("Failed to create GBtrfsFilesystem");
        obj.imp().data.set(fs).unwrap();
        obj
    }

    pub fn data(&self) -> &interface::BtrfsFilesystem {
        self.imp().data.get().unwrap()
    }

    pub fn label(&self) -> Option<&str> {
        self.data().label.as_ref().map(String::as_str)
    }

    pub fn display(&self) -> String {
        if let Some(label) = self.label() {
            label.to_string()
        } else {
            format!("\"{}\"", self.data().devices.get(0).unwrap().display())
        }
    }
}
