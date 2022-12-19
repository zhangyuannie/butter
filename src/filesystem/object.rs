use super::Filesystem as DataFilesystem;
use gtk::{glib, subclass::prelude::*};
use uuid::Uuid;

mod imp {
    use glib::{ParamFlags, ParamSpec, Value};
    use gtk::{glib, prelude::*, subclass::prelude::*};
    use once_cell::sync::{Lazy, OnceCell};

    #[derive(Default)]
    pub struct GFilesystem {
        pub data: OnceCell<super::DataFilesystem>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GFilesystem {
        const NAME: &'static str = "BtrFilesystem";
        type Type = super::GFilesystem;
    }

    impl ObjectImpl for GFilesystem {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::new(
                    "label",
                    None,
                    None,
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
    pub struct GFilesystem(ObjectSubclass<imp::GFilesystem>);
}

impl GFilesystem {
    pub fn new(inner: DataFilesystem) -> Self {
        let ret: Self = glib::Object::new(&[]);
        ret.imp().data.set(inner).unwrap();
        ret
    }

    pub fn data(&self) -> &DataFilesystem {
        self.imp().data.get().unwrap()
    }

    pub fn label(&self) -> &str {
        self.data().label.as_str()
    }

    pub fn uuid(&self) -> Uuid {
        self.data().uuid
    }

    pub fn display(&self) -> String {
        if self.label().is_empty() {
            format!("\"{}\"", self.data().devices[0].display())
        } else {
            self.label().to_string()
        }
    }
}
