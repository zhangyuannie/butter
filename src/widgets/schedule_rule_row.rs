use adw::subclass::prelude::*;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::window::Window;

use super::ScheduleRuleEditDialog;

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
        pub rule: OnceCell<glib::BoxedAnyObject>,
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
            klass.bind_template_instance_callbacks();
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
                    glib::BoxedAnyObject::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "rule" => self
                    .rule
                    .set(value.get::<glib::BoxedAnyObject>().unwrap())
                    .unwrap(),

                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            if let Some(name) = self.rule().name() {
                obj.set_title(name);
            }
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

#[gtk::template_callbacks]
impl ScheduleRuleRow {
    pub fn new(rule: &glib::BoxedAnyObject) -> Self {
        Object::new(&[("rule", rule)]).expect("Failed to create ScheduleRuleRow")
    }

    #[template_callback]
    fn on_activated(&self) {
        let win = self.root().unwrap().downcast::<Window>().unwrap();
        let dialog = ScheduleRuleEditDialog::new(&win.subvolume_manager(), self.imp().rule.get());
        dialog.set_transient_for(Some(&win));
        dialog.show();
    }
}
