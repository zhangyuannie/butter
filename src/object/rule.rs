use std::cell::RefMut;

use butterd::RuleConfig;
use gtk::subclass::prelude::*;
use zbus::zvariant::OwnedObjectPath;

mod imp {
    use std::cell::{Cell, OnceCell, RefCell};

    use butterd::RuleConfig;
    use gtk::{glib, prelude::*, subclass::prelude::*};
    use zbus::zvariant::OwnedObjectPath;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Rule)]
    pub struct Rule {
        pub path: OnceCell<OwnedObjectPath>,
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub is_enabled: Cell<bool>,
        pub config: RefCell<RuleConfig>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Rule {
        const NAME: &'static str = "BtrRule";
        type Type = super::Rule;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Rule {}
}

gtk::glib::wrapper! {
    pub struct Rule(ObjectSubclass<imp::Rule>);
}

impl Default for Rule {
    fn default() -> Self {
        let ret: Self = gtk::glib::Object::new();
        ret.imp().path.set(Default::default());
        ret
    }
}

impl Rule {
    pub fn new(path: OwnedObjectPath, name: String, is_enabled: bool, config: RuleConfig) -> Self {
        let ret: Self = gtk::glib::Object::new();
        let imp = ret.imp();
        imp.path.set(path);
        imp.name.replace(name);
        imp.is_enabled.set(is_enabled);
        imp.config.replace(config);
        ret
    }

    pub fn deep_clone(&self) -> Self {
        Self::new(
            self.object_path().clone(),
            self.name(),
            self.is_enabled(),
            self.config().clone(),
        )
    }

    pub fn config(&self) -> RefMut<'_, RuleConfig> {
        self.imp().config.borrow_mut()
    }

    pub fn object_path(&self) -> &OwnedObjectPath {
        &self.imp().path.get().unwrap()
    }
}
