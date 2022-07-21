use adw::subclass::prelude::*;
use gtk::glib::{Binding, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, Label};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/label_cell.ui")]
    pub struct LabelCell {
        #[template_child]
        pub label: TemplateChild<Label>,
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LabelCell {
        const NAME: &'static str = "LabelCell";
        type ParentType = adw::Bin;
        type Type = super::LabelCell;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LabelCell {}
    impl WidgetImpl for LabelCell {}
    impl BinImpl for LabelCell {}
}

glib::wrapper! {
    pub struct LabelCell(ObjectSubclass<imp::LabelCell>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for LabelCell {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelCell {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create LabelCell")
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
