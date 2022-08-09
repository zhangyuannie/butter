use adw::subclass::prelude::*;
use butter::config;
use butter::json_file::JsonFile;
use butter::schedule::Schedule;
use gtk::glib::{BoxedAnyObject, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use std::path::PathBuf;

use crate::subvolume::SubvolumeManager;

mod imp {
    use std::cell::Ref;

    use butter::{json_file::JsonFile, schedule::Schedule};
    use glib::once_cell::sync::{Lazy, OnceCell};
    use gtk::glib::{ParamSpec, Value};

    use crate::subvolume::SubvolumeManager;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_rule_edit_dialog.ui")]
    pub struct ScheduleRuleEditDialog {
        #[template_child]
        pub save_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub hourly_cell: TemplateChild<gtk::Adjustment>,
        #[template_child]
        pub daily_cell: TemplateChild<gtk::Adjustment>,
        #[template_child]
        pub weekly_cell: TemplateChild<gtk::Adjustment>,
        #[template_child]
        pub monthly_cell: TemplateChild<gtk::Adjustment>,
        #[template_child]
        pub yearly_cell: TemplateChild<gtk::Adjustment>,
        pub client: OnceCell<SubvolumeManager>,
        pub prev_path: OnceCell<PathBuf>,
    }

    impl ScheduleRuleEditDialog {
        pub fn is_new(&self) -> bool {
            self.prev_path.get().is_none()
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
                        "client",
                        "client",
                        "client",
                        SubvolumeManager::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "rule",
                        "rule",
                        "rule",
                        glib::BoxedAnyObject::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "client" => self
                    .client
                    .set(value.get::<SubvolumeManager>().unwrap())
                    .unwrap(),

                "rule" => {
                    let value: Option<&BoxedAnyObject> = value.get().unwrap();
                    if let Some(boxed) = value {
                        let rule: Ref<JsonFile<Schedule>> = boxed.borrow();
                        self.prev_path.set(rule.path.clone()).unwrap();
                        self.name_entry.set_text(rule.name().unwrap());
                        self.hourly_cell.set_value(rule.data.keep_hourly as f64);
                        self.daily_cell.set_value(rule.data.keep_daily as f64);
                        self.weekly_cell.set_value(rule.data.keep_weekly as f64);
                        self.monthly_cell.set_value(rule.data.keep_monthly as f64);
                        self.yearly_cell.set_value(rule.data.keep_yearly as f64);
                    } else {
                        self.hourly_cell.set_value(24.0);
                        self.daily_cell.set_value(30.0);
                        self.monthly_cell.set_value(24.0);
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            if self.is_new() {
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
    pub fn new(client: &SubvolumeManager, rule: Option<&BoxedAnyObject>) -> Self {
        Object::new(&[("client", client), ("rule", &rule)])
            .expect("Failed to create ScheduleRuleEditDialog")
    }

    #[template_callback]
    fn on_save_button_clicked(&self) {
        let imp = self.imp();
        let client = imp.client.get().unwrap();
        assert!(imp.name_entry.text().len() > 0);
        let new_path = PathBuf::from(format!(
            "{}/schedules/{}.json",
            config::PKGSYSCONFDIR,
            imp.name_entry.text()
        ));
        match imp.prev_path.get() {
            Some(prev_path) => {
                println!("Opening '{}'", prev_path.display());
                let mut file = JsonFile::<Schedule>::open(prev_path.clone()).unwrap();
                file.data.keep_hourly = imp.hourly_cell.value() as u32;
                file.data.keep_daily = imp.daily_cell.value() as u32;
                file.data.keep_weekly = imp.weekly_cell.value() as u32;
                file.data.keep_monthly = imp.monthly_cell.value() as u32;
                file.data.keep_yearly = imp.yearly_cell.value() as u32;
                client.flush_schedule(file).unwrap();
                if prev_path != &new_path {
                    println!(
                        "Renaming '{}' to '{}'",
                        prev_path.display(),
                        new_path.display()
                    );
                    client.fs_rename(prev_path.clone(), new_path).unwrap();
                }
            }
            None => {
                let file = JsonFile {
                    path: new_path,
                    data: Schedule {
                        keep_hourly: imp.hourly_cell.value() as u32,
                        keep_daily: imp.daily_cell.value() as u32,
                        keep_weekly: imp.weekly_cell.value() as u32,
                        keep_monthly: imp.monthly_cell.value() as u32,
                        keep_yearly: imp.yearly_cell.value() as u32,
                        ..Default::default()
                    },
                };
                client.flush_schedule(file).unwrap();
            }
        };
    }
}
