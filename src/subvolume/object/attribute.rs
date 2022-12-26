use gtk::prelude::*;

use super::{sorter::GSubvolumeCreatedSorter, GSubvolume};

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
    pub const NAME: &'static str = "name";
    pub const PATH: &'static str = "path";
    pub const PARENT_PATH: &'static str = "parent-path";
    pub const CREATED: &'static str = "created";
    pub const UUID: &'static str = "uuid";

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
