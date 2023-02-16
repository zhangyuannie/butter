use super::Filesystem as DataFilesystem;
use gtk::{glib, subclass::prelude::*};
use uuid::Uuid;

mod imp {
    use glib::{ParamSpec, Value};
    use gtk::{glib, prelude::*, subclass::prelude::*};
    use once_cell::sync::OnceCell;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::GFilesystem)]
    pub struct GFilesystem {
        #[property(name = "label", get, type = String, member = label)]
        pub data: OnceCell<super::DataFilesystem>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GFilesystem {
        const NAME: &'static str = "BtrFilesystem";
        type Type = super::GFilesystem;
    }

    impl ObjectImpl for GFilesystem {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }
        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            Self::derived_property(self, id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct GFilesystem(ObjectSubclass<imp::GFilesystem>);
}

impl GFilesystem {
    pub fn new(inner: DataFilesystem) -> Self {
        let ret: Self = glib::Object::new();
        ret.imp().data.set(inner).unwrap();
        ret
    }

    pub fn data(&self) -> &DataFilesystem {
        self.imp().data.get().unwrap()
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
