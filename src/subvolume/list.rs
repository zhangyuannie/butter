use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use indexmap::IndexMap;
use std::cell::RefCell;
use uuid::Uuid;

use crate::subvolume::GSubvolume;

mod imp {

    use super::*;

    #[derive(Default)]
    pub struct SubvolList {
        pub subvols: RefCell<IndexMap<Uuid, GSubvolume>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubvolList {
        const NAME: &'static str = "SubvolumeList";
        type Type = super::SubvolList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SubvolList {}

    impl ListModelImpl for SubvolList {
        fn item_type(&self) -> glib::Type {
            GSubvolume::static_type()
        }
        fn n_items(&self) -> u32 {
            self.subvols.borrow().len() as u32
        }
        fn item(&self, position: u32) -> Option<glib::Object> {
            let subvols = self.subvols.borrow();

            subvols
                .get_index(position as usize)
                .map(|(_, subvol)| subvol.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct SubvolList(ObjectSubclass<imp::SubvolList>)
        @implements gio::ListModel;
}

impl SubvolList {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn by_id(&self, id: &Uuid) -> Option<GSubvolume> {
        let subvols = self.imp().subvols.borrow();
        subvols.get(id).and_then(|subvol| Some(subvol.clone()))
    }

    pub fn clear(&self) {
        let mut subvols = self.imp().subvols.borrow_mut();
        let removed = subvols.len();
        subvols.clear();
        drop(subvols);

        self.items_changed(0, removed as u32, 0);
    }

    pub fn insert(&self, subvol: GSubvolume) -> Option<GSubvolume> {
        let mut subvols = self.imp().subvols.borrow_mut();
        let (idx, ret) = subvols.insert_full(subvol.uuid(), subvol);
        drop(subvols);

        self.items_changed(idx as u32, if ret.is_some() { 1 } else { 0 }, 1);

        ret
    }
}

impl Default for SubvolList {
    fn default() -> Self {
        Self::new()
    }
}
