mod list;
pub use list::SubvolList;
mod object;
pub use object::{Attribute, GSubvolume};

use std::path::PathBuf;

use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zbus::zvariant::{Optional, Type};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Type)]
pub struct Subvolume {
    pub subvol_path: PathBuf,
    pub mount_path: Optional<PathBuf>,
    pub uuid: Uuid,
    pub id: u64,
    pub created_unix_secs: i64,
    pub snapshot_source_uuid: Optional<Uuid>,
}

impl Default for Subvolume {
    fn default() -> Self {
        Self {
            subvol_path: Default::default(),
            mount_path: Optional::from(None),
            uuid: Default::default(),
            id: Default::default(),
            created_unix_secs: Default::default(),
            snapshot_source_uuid: Optional::from(None),
        }
    }
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
