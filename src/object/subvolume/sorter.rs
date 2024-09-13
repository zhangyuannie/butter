use gtk::glib;

mod created_imp {
    use gtk::{glib, prelude::*, subclass::prelude::*};

    use crate::object::Subvolume;

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
            let e1 = item1.downcast_ref::<Subvolume>().unwrap().created();
            let e2 = item2.downcast_ref::<Subvolume>().unwrap().created();
            e1.cmp(&e2).into()
        }

        fn order(&self) -> gtk::SorterOrder {
            gtk::SorterOrder::Partial
        }
    }
}

glib::wrapper! {
    pub struct GSubvolumeCreatedSorter(ObjectSubclass<created_imp::GSubvolumeCreatedSorter>)
        @extends gtk::Sorter;
}

impl GSubvolumeCreatedSorter {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
