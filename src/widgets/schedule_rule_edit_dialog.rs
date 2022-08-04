use adw::subclass::prelude::*;
use butter::json_file::JsonFile;
use butter::schedule::Schedule;
use gtk::glib::{BoxedAnyObject, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use std::cell::{Cell, Ref};

    use butter::{json_file::JsonFile, schedule::Schedule};
    use glib::once_cell::sync::{Lazy, OnceCell};
    use gtk::glib::{ParamSpec, Value};

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_rule_edit_dialog.ui")]
    pub struct ScheduleRuleEditDialog {
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        pub rule: OnceCell<glib::BoxedAnyObject>,
        pub is_new: Cell<bool>,
    }

    impl ScheduleRuleEditDialog {
        pub fn rule(&self) -> Ref<JsonFile<Schedule>> {
            self.rule.get().unwrap().borrow()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScheduleRuleEditDialog {
        const NAME: &'static str = "ScheduleRuleEditDialog";
        type ParentType = gtk::Window;
        type Type = super::ScheduleRuleEditDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScheduleRuleEditDialog {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "rule",
                        "rule",
                        "rule",
                        glib::BoxedAnyObject::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-new",
                        "is-new",
                        "is-new",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "rule" => self.rule.set(value.get().unwrap()).unwrap(),
                "is-new" => self.is_new.set(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            if self.is_new.get() {
                self.save_button.set_label("Create");
                obj.set_title(Some("New Rule"));
            } else {
                self.save_button.set_label("Apply");
                obj.set_title(Some("Edit Rule"));
            }
        }
    }
    impl WidgetImpl for ScheduleRuleEditDialog {}
    impl WindowImpl for ScheduleRuleEditDialog {}
}

glib::wrapper! {
    pub struct ScheduleRuleEditDialog(ObjectSubclass<imp::ScheduleRuleEditDialog>)
    @extends gtk::Window, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl ScheduleRuleEditDialog {
    pub fn new(rule: Option<&BoxedAnyObject>) -> Self {
        let inner = if let Some(rule) = rule {
            rule.borrow::<JsonFile<Schedule>>().clone()
        } else {
            JsonFile::<Schedule>::default()
        };
        Object::new(&[
            ("rule", &BoxedAnyObject::new(inner)),
            ("is-new", &rule.is_none()),
        ])
        .expect("Failed to create ScheduleRuleEditDialog")
    }

    #[template_callback]
    fn on_save_button_clicked(&self) {
    }
}
