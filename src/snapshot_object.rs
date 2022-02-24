use glib::Object;
use gtk::{glib, prelude::ObjectExt};
use serde::Deserialize;

mod imp {
    use std::{cell::RefCell, path::Path, rc::Rc};

    use gtk::{
        glib::{
            self, once_cell::sync::Lazy, Binding, ParamFlags, ParamSpec, ParamSpecString, Value,
        },
        prelude::*,
        subclass::prelude::*,
    };

    #[derive(Default)]
    pub struct SnapshotObject {
        pub data: Rc<RefCell<super::SnapshotData>>,
        pub binding: RefCell<Option<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SnapshotObject {
        const NAME: &'static str = "SnapshotObject";
        type Type = super::SnapshotObject;
    }

    impl ObjectImpl for SnapshotObject {
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
                    self.data.borrow_mut().parent_path = input_value;
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

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => Path::new(&self.data.borrow().path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .to_value(),
                "path" => self.data.borrow().path.to_value(),
                "parent-path" => self.data.borrow().parent_path.to_value(),
                "creation-time" => self.data.borrow().creation_time.to_value(),
                "absolute-path" => self.data.borrow().absolute_path.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct SnapshotObject(ObjectSubclass<imp::SnapshotObject>);
}

impl SnapshotObject {
    pub fn new(
        path: String,
        absolute_path: String,
        parent_path: String,
        creation_time: String,
    ) -> Self {
        Object::new(&[
            ("path", &path),
            ("parent-path", &parent_path),
            ("creation-time", &creation_time),
            ("absolute-path", &absolute_path),
        ])
        .expect("Failed to create SnapshotObject")
    }

    pub fn absolute_path(&self) -> String {
        self.property("absolute-path")
    }
}

impl From<SnapshotData> for SnapshotObject {
    fn from(item: SnapshotData) -> Self {
        SnapshotObject::new(
            item.path,
            item.absolute_path,
            item.parent_path,
            item.creation_time,
        )
    }
}

#[derive(Default, Deserialize)]
pub struct SnapshotData {
    pub path: String,
    pub absolute_path: String,
    pub parent_path: String,
    pub creation_time: String,
}
