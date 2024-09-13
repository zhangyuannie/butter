pub mod attribute;
pub mod list;
pub mod sorter;

use attribute::Attribute;

use std::{borrow::Cow, path::Path};

use gtk::{glib, subclass::prelude::*};
use uuid::Uuid;

mod imp {
    use std::{cell::OnceCell, sync::LazyLock};

    use glib::{ParamSpec, Value};
    use gtk::{glib, prelude::*, subclass::prelude::*};

    use crate::object::attribute::Attribute;

    #[derive(Default)]
    pub struct Subvolume {
        pub data: OnceCell<butterd::Subvolume>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Subvolume {
        const NAME: &'static str = "BtrSubvolume";
        type Type = super::Subvolume;
    }

    impl ObjectImpl for Subvolume {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: LazyLock<Vec<ParamSpec>> = LazyLock::new(|| {
                vec![
                    glib::ParamSpecString::builder(Attribute::NAME)
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder(Attribute::PATH)
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder(Attribute::PARENT_PATH)
                        .read_only()
                        .build(),
                    glib::ParamSpecBoxed::builder::<glib::DateTime>(Attribute::CREATED)
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder(Attribute::UUID)
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            let obj = self.obj();
            match pspec.name() {
                Attribute::NAME => obj.name().to_value(),
                Attribute::PATH => obj.attribute_str(Attribute::Path).to_value(),
                Attribute::PARENT_PATH => obj.attribute_str(Attribute::ParentPath).to_value(),
                Attribute::CREATED => obj.created().to_value(),
                Attribute::UUID => obj.attribute_str(Attribute::Uuid).to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Subvolume(ObjectSubclass<imp::Subvolume>);
}

impl Subvolume {
    pub fn new(subvol: butterd::Subvolume) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().data.set(subvol).unwrap();
        obj
    }

    fn data(&self) -> &butterd::Subvolume {
        self.imp().data.get().unwrap()
    }

    pub fn uuid(&self) -> Uuid {
        self.data().uuid.into()
    }

    pub fn name(&self) -> Cow<str> {
        self.subvol_path().file_name().unwrap().to_string_lossy()
    }

    pub fn subvol_path(&self) -> &Path {
        self.data().root_path.as_path()
    }

    pub fn mount_path(&self) -> Option<&Path> {
        self.data().paths.first().map(|p| p.as_path())
    }

    /// Subvolume that are generally stable and should not be deleted
    pub fn is_protected(&self) -> bool {
        self.data().is_likely_primary()
    }

    pub fn created(&self) -> glib::DateTime {
        glib::DateTime::from_unix_local(self.data().created_unix_secs).unwrap()
    }

    pub fn attribute_str(&self, attribute: Attribute) -> String {
        match attribute {
            Attribute::Name => self.name().to_string(),
            Attribute::Path => self.subvol_path().to_string_lossy().to_string(),
            Attribute::ParentPath => self
                .data()
                .created_from_root_path
                .clone()
                .map_or(String::new(), |p| p.as_path().to_string_lossy().into()),
            Attribute::Created => self.created().format("%c").unwrap().into(),
            Attribute::Uuid => self.uuid().to_string(),
        }
    }
}
