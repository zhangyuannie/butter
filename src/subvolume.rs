mod list;
pub use list::SubvolList;

mod g_btrfs_filesystem;
mod subvolume_manager;
pub use g_btrfs_filesystem::GBtrfsFilesystem;

mod created_sorter;
pub use created_sorter::GSubvolumeCreatedSorter;

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
        const NAME: &'static str = "Subvolume";
        type Type = super::GSubvolume;
    }

    impl ObjectImpl for GSubvolume {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "name",
                        "Name",
                        "Filename",
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        "path",
                        "Path",
                        "Absolute Path",
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecString::new(
                        "parent-path",
                        "Parent Path",
                        "Path of the subvolume this is a snapshot of",
                        None,
                        ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoxed::new(
                        "created",
                        "Created",
                        "Creation Time",
                        glib::DateTime::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => obj.name().to_value(),
                "path" => obj.path().to_str().to_value(),
                "parent-path" => obj
                    .parent()
                    .map_or("".to_value(), |parent| parent.path().to_str().to_value()),
                "created" => obj.g_created().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct GSubvolume(ObjectSubclass<imp::GSubvolume>);
}

impl GSubvolume {
    pub fn new(subvol: interface::Subvolume) -> Self {
        let obj: Self = Object::new(&[]).expect("Failed to create GSubvolume");
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
}
