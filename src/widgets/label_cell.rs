use adw::subclass::prelude::*;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, Label};

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/label_cell.ui")]
    pub struct LabelCell {
        #[template_child]
        pub label: TemplateChild<Label>,
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

    pub fn label(&self) -> &gtk::Label {
        &self.imp().label
    }
}
