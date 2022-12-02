use adw::subclass::prelude::*;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate, Label};

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/subvolume_label_cell.ui")]
    pub struct SubvolumeLabelCell {
        #[template_child]
        pub label: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubvolumeLabelCell {
        const NAME: &'static str = "SubvolumeLabelCell";
        type ParentType = adw::Bin;
        type Type = super::SubvolumeLabelCell;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SubvolumeLabelCell {}
    impl WidgetImpl for SubvolumeLabelCell {}
    impl BinImpl for SubvolumeLabelCell {}
}

glib::wrapper! {
    pub struct SubvolumeLabelCell(ObjectSubclass<imp::SubvolumeLabelCell>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for SubvolumeLabelCell {
    fn default() -> Self {
        Self::new()
    }
}

impl SubvolumeLabelCell {
    pub fn new() -> Self {
        Object::new(&[])
    }

    pub fn label(&self) -> &gtk::Label {
        &self.imp().label
    }
}
