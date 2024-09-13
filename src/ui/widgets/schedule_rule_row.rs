use gtk::glib;
use gtk::subclass::prelude::*;

use crate::object::Rule;

mod imp {

    use std::cell::OnceCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gtk::{
        glib::{self, ParamSpec, Value},
        CompositeTemplate, TemplateChild,
    };

    use crate::object::Rule;

    #[derive(Default, CompositeTemplate, glib::Properties)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_rule_row.ui")]
    #[properties(wrapper_type = super::ScheduleRuleRow)]
    pub struct ScheduleRuleRow {
        #[template_child]
        pub switch: TemplateChild<gtk::Switch>,
        #[property(get, set, construct_only)]
        pub rule: OnceCell<Rule>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScheduleRuleRow {
        const NAME: &'static str = "ScheduleRuleRow";
        type ParentType = adw::ActionRow;
        type Type = super::ScheduleRuleRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScheduleRuleRow {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            Self::derived_property(self, id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_title(obj.rule().name().as_ref());
            self.switch.set_active(obj.rule().is_enabled());
        }
    }
    impl WidgetImpl for ScheduleRuleRow {}
    impl ListBoxRowImpl for ScheduleRuleRow {}
    impl PreferencesRowImpl for ScheduleRuleRow {}
    impl ActionRowImpl for ScheduleRuleRow {}
}

glib::wrapper! {
    pub struct ScheduleRuleRow(ObjectSubclass<imp::ScheduleRuleRow>)
    @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ScheduleRuleRow {
    pub fn new(rule: &Rule) -> Self {
        glib::Object::builder().property("rule", rule).build()
    }

    pub fn switch(&self) -> &gtk::Switch {
        &self.imp().switch
    }
}
