mod attribute;
mod sorter;
pub use attribute::Attribute;
use once_cell::sync::Lazy;

use std::{
    borrow::Cow,
    collections::HashSet,
    path::{Path, PathBuf},
};

use gtk::{glib, subclass::prelude::*};
use uuid::Uuid;

use crate::subvolume::Subvolume;

mod imp {
    use glib::{ParamFlags, ParamSpec, Value, WeakRef};
    use gtk::{glib, prelude::*, subclass::prelude::*};
    use once_cell::sync::{Lazy, OnceCell};

    use crate::subvolume::{Attribute, Subvolume};

    #[derive(Default)]
    pub struct GSubvolume {
        pub data: OnceCell<Subvolume>,
        pub parent: WeakRef<super::GSubvolume>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GSubvolume {
        const NAME: &'static str = "BtrSubvolume";
        type Type = super::GSubvolume;
    }

    impl ObjectImpl for GSubvolume {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        Attribute::NAME,
                        None,
                        None,
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        Attribute::PATH,
                        None,
                        None,
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        Attribute::PARENT_PATH,
                        None,
                        None,
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoxed::new(
                        Attribute::CREATED,
                        None,
                        None,
                        glib::DateTime::static_type(),
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        Attribute::UUID,
                        None,
                        None,
                        None,
                        ParamFlags::READABLE,
                    ),
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
    pub struct GSubvolume(ObjectSubclass<imp::GSubvolume>);
}

impl GSubvolume {
    pub fn new(subvol: Subvolume) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().data.set(subvol).unwrap();
        obj
    }

    fn data(&self) -> &Subvolume {
        self.imp().data.get().unwrap()
    }

    pub fn uuid(&self) -> Uuid {
        self.data().uuid
    }

    pub fn name(&self) -> Cow<str> {
        self.subvol_path().file_name().unwrap().to_string_lossy()
    }

    pub fn subvol_path(&self) -> &Path {
        &self.data().subvol_path
    }

    pub fn mount_path(&self) -> Option<&Path> {
        self.data()
            .mount_path
            .as_ref()
            .and_then(|p| Some(p.as_path()))
    }

    /// Subvolume that are generally stable and should not be deleted
    pub fn is_protected(&self) -> bool {
        // some hardcoded paths that are extremly often to be their own subvolume
        // and important for the system. This is not meant to be exhaustive
        static IMPORTANT_PATHS: Lazy<HashSet<PathBuf>> = Lazy::new(|| {
            HashSet::from([
                PathBuf::from("/"),
                PathBuf::from("/boot"),
                PathBuf::from("/home"),
                PathBuf::from("/var"),
            ])
        });

        let ret = self.data().is_mountpoint
            || self.data().snapshot_source_uuid.is_none()
            || self.data().subvol_path == Path::new("/");
        if ret {
            return ret;
        }
        if let Some(mnt) = &*self.data().mount_path {
            return IMPORTANT_PATHS.contains(mnt);
        }
        false
    }

    pub fn created(&self) -> glib::DateTime {
        glib::DateTime::from_unix_local(self.data().created_unix_secs).unwrap()
    }

    pub fn parent_uuid(&self) -> Option<Uuid> {
        *self.data().snapshot_source_uuid
    }

    pub fn parent(&self) -> Option<GSubvolume> {
        self.imp().parent.upgrade()
    }

    pub fn set_parent(&self, subvol: Option<&GSubvolume>) {
        self.imp().parent.set(subvol)
    }

    pub fn attribute_str(&self, attribute: Attribute) -> String {
        match attribute {
            Attribute::Name => self.name().to_string(),
            Attribute::Path => self.subvol_path().to_string_lossy().to_string(),
            Attribute::ParentPath => self.parent().map_or("".to_string(), |parent| {
                parent.subvol_path().to_string_lossy().to_string()
            }),
            Attribute::Created => self.created().format("%c").unwrap().into(),
            Attribute::Uuid => self.uuid().to_string(),
        }
    }
}
