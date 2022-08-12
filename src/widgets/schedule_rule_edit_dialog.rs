use adw::prelude::*;
use adw::subclass::prelude::*;
use butter::config;
use butter::schedule::ScheduleSubvolume;
use gtk::glib::Object;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use std::path::PathBuf;

use crate::schedule_repo::{ScheduleObject, ScheduleRepo};

mod imp {
    use std::cell::{Ref, RefCell};

    use butter::{
        json_file::JsonFile,
        schedule::{Schedule, ScheduleSubvolume},
    };
    use glib::once_cell::sync::{Lazy, OnceCell};
    use gtk::{
        gio::ListStore,
        glib::{ParamSpec, Value},
    };

    use crate::widgets::FileChooserEntry;

    use super::*;

    pub struct ScheduleTargetModel(ListStore);
    impl Default for ScheduleTargetModel {
        fn default() -> Self {
            Self(ListStore::new(glib::BoxedAnyObject::static_type()))
        }
    }

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
        #[template_child]
        pub remove_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub header_prefix_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub subvolume_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub add_subvolume_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub subvol_path_entry: TemplateChild<FileChooserEntry>,
        #[template_child]
        pub target_dir_entry: TemplateChild<FileChooserEntry>,
        pub repo: OnceCell<ScheduleRepo>,
        pub schedule: OnceCell<ScheduleObject>,
        pub subvolumes: RefCell<Vec<ScheduleSubvolume>>,
    }

    impl ScheduleRuleEditDialog {
        pub fn is_new(&self) -> bool {
            self.schedule.get().is_none()
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
                        "repo",
                        "repo",
                        "repo",
                        ScheduleRepo::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "rule",
                        "rule",
                        "rule",
                        ScheduleObject::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "repo" => self.repo.set(value.get().unwrap()).unwrap(),

                "rule" => {
                    let maybe_schedule: Option<&ScheduleObject> = value.get().unwrap();
                    if let Some(schedule) = maybe_schedule {
                        self.schedule.set(schedule.clone()).unwrap();
                        let rule: Ref<JsonFile<Schedule>> = schedule.borrow();
                        self.name_entry.set_text(rule.name().unwrap());
                        self.hourly_cell.set_value(rule.data.keep_hourly as f64);
                        self.daily_cell.set_value(rule.data.keep_daily as f64);
                        self.weekly_cell.set_value(rule.data.keep_weekly as f64);
                        self.monthly_cell.set_value(rule.data.keep_monthly as f64);
                        self.yearly_cell.set_value(rule.data.keep_yearly as f64);
                        self.subvolumes
                            .borrow_mut()
                            .extend_from_slice(&rule.data.subvolumes);
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
                self.remove_group.set_visible(false);
            } else {
                self.save_button.set_label("Apply");
                obj.set_title(Some("Edit Rule"));
            }
            obj.reload_subvolume_list();
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
    pub fn new(repo: &ScheduleRepo, rule: Option<&ScheduleObject>) -> Self {
        Object::new(&[("repo", repo), ("rule", &rule)])
            .expect("Failed to create ScheduleRuleEditDialog")
    }

    fn reload_subvolume_list(&self) {
        let imp = self.imp();
        if let Some(mut child) = imp.subvolume_list.first_child() {
            while let Some(sibling) = child.next_sibling() {
                imp.subvolume_list.remove(&child);
                child = sibling;
            }
        }
        for (idx, subvol) in imp.subvolumes.borrow().iter().enumerate() {
            let remove_btn = gtk::Button::builder()
                .icon_name("list-remove-symbolic")
                .valign(gtk::Align::Center)
                .css_classes(vec!["flat".to_string(), "circular".to_string()])
                .build();
            remove_btn.connect_clicked(glib::clone!(@weak self as dialog => move |_| {
                dialog.imp().subvolumes.borrow_mut().remove(idx);
                dialog.reload_subvolume_list();
            }));
            let row = adw::ActionRow::builder()
                .title(&subvol.path.to_string_lossy())
                .subtitle(&subvol.target_dir.to_string_lossy())
                .build();
            row.add_prefix(&remove_btn);
            imp.subvolume_list.insert(&row, idx as i32);
        }
    }

    #[template_callback]
    fn on_save_button_clicked(&self) {
        let imp = self.imp();
        let repo = imp.repo.get().unwrap();
        assert!(imp.name_entry.text().len() > 0);
        let new_path = PathBuf::from(format!(
            "{}/schedules/{}.json",
            config::PKGSYSCONFDIR,
            imp.name_entry.text()
        ));

        let obj = match imp.schedule.get() {
            Some(obj) => obj.clone(),
            None => ScheduleObject::default(),
        };
        obj.set_path(new_path);
        {
            let mut obj = obj.borrow_mut();
            obj.data.keep_hourly = imp.hourly_cell.value() as u32;
            obj.data.keep_daily = imp.daily_cell.value() as u32;
            obj.data.keep_weekly = imp.weekly_cell.value() as u32;
            obj.data.keep_monthly = imp.monthly_cell.value() as u32;
            obj.data.keep_yearly = imp.yearly_cell.value() as u32;
            obj.data.subvolumes.clear();
            obj.data
                .subvolumes
                .extend_from_slice(&imp.subvolumes.borrow());
        }
        repo.persist(&obj).unwrap();
        repo.sync().unwrap();
        self.close();
    }

    #[template_callback]
    fn on_remove_button_clicked(&self) {
        let imp = self.imp();
        if let Some(schedule) = imp.schedule.get() {
            let repo = imp.repo.get().unwrap();
            repo.delete(schedule).unwrap();
            repo.sync().unwrap();
            self.close();
        }
    }

    #[template_callback]
    fn on_back_button_clicked(&self) {
        self.imp().stack.set_visible_child_name("main");
        self.imp()
            .header_prefix_stack
            .set_visible_child_name("cancel");
    }

    #[template_callback]
    fn show_subvolumes(&self) {
        self.imp().stack.set_visible_child_name("subvol");
        self.imp()
            .header_prefix_stack
            .set_visible_child_name("back");
    }

    #[template_callback]
    fn on_add_subvolume_clicked(&self) {
        let imp = self.imp();
        if imp.subvol_path_entry.text().len() > 0 && imp.target_dir_entry.text().len() > 0 {
            imp.subvolumes.borrow_mut().push(ScheduleSubvolume {
                path: imp.subvol_path_entry.text().to_string().into(),
                target_dir: imp.target_dir_entry.text().to_string().into(),
            });
            self.reload_subvolume_list();
            imp.subvol_path_entry.set_text("");
            imp.target_dir_entry.set_text("");
            imp.add_subvolume_row.set_expanded(false);
        }
    }
}
