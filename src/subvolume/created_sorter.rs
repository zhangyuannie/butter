use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
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
        fn compare(
            &self,
            _sorter: &Self::Type,
            item1: &glib::Object,
            item2: &glib::Object,
        ) -> gtk::Ordering {
            let e1 = item1.downcast_ref::<GSubvolume>().unwrap().created();
            let e2 = item2.downcast_ref::<GSubvolume>().unwrap().created();
            e1.cmp(&e2).into()
        }

        fn order(&self, _sorter: &Self::Type) -> gtk::SorterOrder {
            gtk::SorterOrder::Partial
        }
    }
}

glib::wrapper! {
    pub struct GSubvolumeCreatedSorter(ObjectSubclass<imp::GSubvolumeCreatedSorter>)
        @extends gtk::Sorter;
}

impl GSubvolumeCreatedSorter {
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}
