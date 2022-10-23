mod list;
pub use list::SubvolList;

mod g_btrfs_filesystem;
mod subvolume_manager;
pub use g_btrfs_filesystem::GBtrfsFilesystem;

use butter::daemon::interface;
pub use subvolume_manager::SubvolumeManager;
use uuid::Uuid;

use std::{path::Path, time::SystemTime};

use glib::Object;
use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    use glib::once_cell::sync::OnceCell;

    use gtk::glib::{self, once_cell::sync::Lazy, ParamFlags, ParamSpec, Value, WeakRef};

    #[derive(Default)]
    pub struct GSubvolume {
        pub data: OnceCell<interface::Subvolume>,
        pub parent: WeakRef<super::GSubvolume>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GSubvolume {
        const NAME: &'static str = "SubvolumeObject";
        type Type = super::GSubvolume;
    }

    impl ObjectImpl for GSubvolume {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        Attribute::NAME,
                        Attribute::NAME,
                        Attribute::NAME,
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        Attribute::PATH,
                        Attribute::PATH,
                        Attribute::PATH,
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        Attribute::PARENT_PATH,
                        Attribute::PARENT_PATH,
                        Attribute::PARENT_PATH,
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoxed::new(
                        Attribute::CREATED,
                        Attribute::CREATED,
                        Attribute::CREATED,
                        glib::DateTime::static_type(),
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        Attribute::UUID,
                        Attribute::UUID,
                        Attribute::UUID,
                        None,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            let obj = self.instance();
            match pspec.name() {
                Attribute::NAME => obj.name().to_value(),
                Attribute::PATH => obj.attribute_str(Attribute::Path).to_value(),
                Attribute::PARENT_PATH => obj.attribute_str(Attribute::ParentPath).to_value(),
                Attribute::CREATED => obj.g_created().to_value(),
                Attribute::UUID => obj.attribute_str(Attribute::Uuid).to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct GSubvolume(ObjectSubclass<imp::GSubvolume>);
}

#[derive(Debug, Clone, Copy)]
pub enum Attribute {
    /// Filename
    Name,
    /// Absolute path
    Path,
    /// Path of the subvolume this is a snapshot of
    ParentPath,
    /// Creation time
    Created,
    Uuid,
}

impl Attribute {
    const NAME: &'static str = "name";
    const PATH: &'static str = "path";
    const PARENT_PATH: &'static str = "parent-path";
    const CREATED: &'static str = "created";
    const UUID: &'static str = "uuid";

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Name => Self::NAME,
            Self::Path => Self::PATH,
            Self::ParentPath => Self::PARENT_PATH,
            Self::Created => Self::CREATED,
            Self::Uuid => Self::UUID,
        }
    }

    pub fn sorter(&self) -> gtk::Sorter {
        match self {
            Attribute::Created => GSubvolumeCreatedSorter::new().upcast(),
            _ => gtk::StringSorter::new(Some(&gtk::PropertyExpression::new(
                GSubvolume::static_type(),
                None::<&gtk::Expression>,
                self.as_str(),
            )))
            .upcast(),
        }
    }
}

impl GSubvolume {
    pub fn new(subvol: interface::Subvolume) -> Self {
        let obj: Self = Object::new(&[]);
        obj.imp().data.set(subvol).unwrap();
        obj
    }

    fn data(&self) -> &interface::Subvolume {
        self.imp().data.get().unwrap()
    }

    pub fn uuid(&self) -> Uuid {
        self.data().uuid
    }

    pub fn name(&self) -> &str {
        self.data().path.file_name().unwrap().to_str().unwrap()
    }

    pub fn path(&self) -> &Path {
        self.data().path.as_path()
    }

    pub fn is_snapshot(&self) -> bool {
        self.data().snapshot_source_uuid.is_some()
    }

    pub fn created(&self) -> SystemTime {
        self.data().created
    }

    pub fn g_created(&self) -> glib::DateTime {
        let created = self
            .data()
            .created
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        glib::DateTime::from_unix_local(created.as_secs() as i64).unwrap()
    }

    pub fn parent_uuid(&self) -> Option<Uuid> {
        self.data().snapshot_source_uuid
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
            Attribute::Path => self.path().to_string_lossy().to_string(),
            Attribute::ParentPath => self.parent().map_or("".to_string(), |parent| {
                parent.path().to_string_lossy().to_string()
            }),
            Attribute::Created => self.g_created().format("%c").unwrap().into(),
            Attribute::Uuid => self.uuid().to_string(),
        }
    }
}

mod created_sorter_imp {
    use crate::subvolume::GSubvolume;

    use super::*;

    #[derive(Default)]
    pub struct GSubvolumeCreatedSorter;

    #[glib::object_subclass]
    impl ObjectSubclass for GSubvolumeCreatedSorter {
        const NAME: &'static str = "GSubvolumeCreatedSorter";
        type ParentType = gtk::Sorter;
        type Type = super::GSubvolumeCreatedSorter;
    }

    impl ObjectImpl for GSubvolumeCreatedSorter {}
    impl SorterImpl for GSubvolumeCreatedSorter {
        fn compare(&self, item1: &glib::Object, item2: &glib::Object) -> gtk::Ordering {
            let e1 = item1.downcast_ref::<GSubvolume>().unwrap().created();
            let e2 = item2.downcast_ref::<GSubvolume>().unwrap().created();
            e1.cmp(&e2).into()
        }

        fn order(&self) -> gtk::SorterOrder {
            gtk::SorterOrder::Partial
        }
    }
}

glib::wrapper! {
    pub struct GSubvolumeCreatedSorter(ObjectSubclass<created_sorter_imp::GSubvolumeCreatedSorter>)
        @extends gtk::Sorter;
}

impl GSubvolumeCreatedSorter {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }
}
