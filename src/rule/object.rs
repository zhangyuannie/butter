use std::borrow::Cow;

use gtk::{glib, subclass::prelude::*};

use super::Rule;

mod imp {
    use gtk::{glib, subclass::prelude::*};
    use once_cell::sync::OnceCell;

    use crate::rule::Rule;

    #[derive(Default)]
    pub struct GRule {
        pub inner: OnceCell<Rule>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GRule {
        const NAME: &'static str = "BtrRule";
        type Type = super::GRule;
    }

    impl ObjectImpl for GRule {}
}

glib::wrapper! {
    pub struct GRule(ObjectSubclass<imp::GRule>);
}

impl GRule {
    pub fn new(inner: Rule) -> Self {
        let ret: Self = glib::Object::new();
        ret.imp().inner.set(inner).unwrap();
        ret
    }

    pub fn inner(&self) -> &Rule {
        self.imp().inner.get().unwrap()
    }

    pub fn name(&self) -> Cow<str> {
        self.inner()
            .path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
    }

    pub fn is_enabled(&self) -> bool {
        self.inner().is_enabled
    }
}
