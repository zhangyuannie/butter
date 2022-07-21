use adw::subclass::prelude::*;
use gtk::{
    glib::{self, Object},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_view.ui")]
    pub struct ScheduleView {
        #[template_child]
        pub rule_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub empty_rule_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub main_rule_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScheduleView {
        const NAME: &'static str = "ScheduleView";
        type ParentType = adw::Bin;
        type Type = super::ScheduleView;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScheduleView {}
    impl WidgetImpl for ScheduleView {}
    impl BinImpl for ScheduleView {}
}

glib::wrapper! {
    pub struct ScheduleView(ObjectSubclass<imp::ScheduleView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ScheduleView {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create ScheduleView")
    }
}
