use gtk::glib;
use gtk::glib::Object;
use gtk::subclass::prelude::*;

use crate::rule::GRule;

mod imp {

    use adw::subclass::prelude::*;
    use adw::traits::PreferencesRowExt;
    use gtk::{
        glib::{self, ParamSpec, Value},
        prelude::*,
        CompositeTemplate, TemplateChild,
    };
    use once_cell::sync::{Lazy, OnceCell};

    use crate::rule::GRule;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_rule_row.ui")]
    pub struct ScheduleRuleRow {
        #[template_child]
        pub switch: TemplateChild<gtk::Switch>,
        pub rule: OnceCell<GRule>,
    }

    impl ScheduleRuleRow {
        pub fn rule(&self) -> &GRule {
            self.rule.get().unwrap()
        }
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
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "rule",
                    None,
                    None,
                    GRule::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "rule" => self.rule.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.instance().set_title(self.rule().name().as_ref());

            self.switch.set_active(self.rule().is_enabled());
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
    pub fn new(rule: &GRule) -> Self {
        Object::new(&[("rule", rule)])
    }

    pub fn switch(&self) -> &gtk::Switch {
        &self.imp().switch
    }
}
