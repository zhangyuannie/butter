use glib::Object;
use gtk::glib;
use gtk::glib::Binding;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use glib::Binding;
    use gtk::{CompositeTemplate, Label};
    use std::cell::RefCell;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/snapshot_column_cell.ui")]
    pub struct SnapshotColumnCell {
        #[template_child]
        pub label: TemplateChild<Label>,
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SnapshotColumnCell {
        const NAME: &'static str = "SnapshotColumnCell";
        type Type = super::SnapshotColumnCell;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SnapshotColumnCell {}
    impl WidgetImpl for SnapshotColumnCell {}
    impl BoxImpl for SnapshotColumnCell {}
}

glib::wrapper! {
    pub struct SnapshotColumnCell(ObjectSubclass<imp::SnapshotColumnCell>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for SnapshotColumnCell {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotColumnCell {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create SnapshotColumnCell")
    }

    pub fn add_binding(&self, binding: Binding) {
        let mut bindings = self.imp().bindings.borrow_mut();
        bindings.push(binding);
    }

    pub fn unbind_all(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }

    pub fn label(&self) -> gtk::Label {
        self.imp().label.get()
    }
}
