mod subvolume_manager;

pub use subvolume_manager::SubvolumeManager;

use std::path::Path;

use glib::Object;
use gtk::{glib, subclass::prelude::*};
use serde::Deserialize;

mod imp {
    use std::{cell::RefCell, rc::Rc};

    use gtk::{
        glib::{self, once_cell::sync::Lazy, ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::*,
        subclass::prelude::*,
    };

    #[derive(Default)]
    pub struct Subvolume {
        pub data: Rc<RefCell<super::SubvolumeData>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Subvolume {
        const NAME: &'static str = "Subvolume";
        type Type = super::Subvolume;
    }

    impl ObjectImpl for Subvolume {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("name", "name", "name", None, ParamFlags::READABLE),
                    ParamSpecString::new("path", "path", "path", None, ParamFlags::READWRITE),
                    ParamSpecString::new(
                        "parent-path",
                        "parent-path",
                        "parent-path",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "creation-time",
                        "creation-time",
                        "creation-time",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "absolute-path",
                        "absolute-path",
                        "absolute-path",
                        None,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "path" => {
                    let input_value = value.get().expect("Value must be String");
                    self.data.borrow_mut().path = input_value;
                }
                "parent-path" => {
                    let input_value = value.get().expect("Value must be String");
                    self.data.borrow_mut().snapshot_source_path = input_value;
                }
                "creation-time" => {
                    let input_value = value.get().expect("Value must be String");
                    self.data.borrow_mut().creation_time = input_value;
                }
                "absolute-path" => {
                    let input_value = value.get().expect("Value must be String");
                    self.data.borrow_mut().absolute_path = input_value;
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => obj.name().to_value(),
                "path" => self.data.borrow().path.to_value(),
                "parent-path" => self.data.borrow().snapshot_source_path.to_value(),
                "creation-time" => self.data.borrow().creation_time.to_value(),
                "absolute-path" => obj.mounted_path().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Subvolume(ObjectSubclass<imp::Subvolume>);
}

impl Subvolume {
    pub fn new(data: SubvolumeData) -> Self {
        Object::new(&[
            ("path", &data.path),
            ("parent-path", &data.snapshot_source_path),
            ("creation-time", &data.creation_time),
            ("absolute-path", &data.absolute_path),
        ])
        .expect("Failed to create Subvolume")
    }

    pub fn name(&self) -> String {
        Path::new(&self.imp().data.borrow().path)
            .file_name()
            .unwrap()
            .to_owned()
            .into_string()
            .unwrap()
    }

    pub fn mounted_path(&self) -> Option<String> {
        self.imp().data.borrow().absolute_path.to_owned()
    }

    pub fn is_snapshot(&self) -> bool {
        self.imp().data.borrow().snapshot_source_path.is_some()
    }
}

impl From<SubvolumeData> for Subvolume {
    fn from(item: SubvolumeData) -> Self {
        Subvolume::new(item)
    }
}

#[derive(Default, Clone, Deserialize, glib::Variant)]
pub struct SubvolumeData {
    pub path: String,
    pub absolute_path: Option<String>,
    pub snapshot_source_path: Option<String>,
    pub creation_time: String,
}
