use adw::subclass::prelude::*;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::schedule_repo::ScheduleObject;

mod imp {
    use std::cell::Ref;

    use adw::traits::PreferencesRowExt;
    use butter::{json_file::JsonFile, schedule::Schedule};
    use glib::once_cell::sync::{Lazy, OnceCell};
    use gtk::glib::{ParamSpec, Value};

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_rule_row.ui")]
    pub struct ScheduleRuleRow {
        #[template_child]
        pub switch: TemplateChild<gtk::Switch>,
        pub rule: OnceCell<ScheduleObject>,
    }

    impl ScheduleRuleRow {
        pub fn rule(&self) -> Ref<JsonFile<Schedule>> {
            self.rule.get().unwrap().borrow()
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
                    "rule",
                    "rule",
                    ScheduleObject::static_type(),
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
            if let Some(name) = self.rule().name() {
                self.instance().set_title(name);
            }
            self.switch.set_active(self.rule().data.is_enabled);
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
    pub fn new(rule: &ScheduleObject) -> Self {
        Object::new(&[("rule", rule)])
    }

    pub fn switch(&self) -> &gtk::Switch {
        &self.imp().switch
    }
}
