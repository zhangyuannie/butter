use butterd::RuleSubvolumeConfig;
use gtk::glib;
use std::path::PathBuf;

use crate::config;
use crate::object::Rule;
use crate::ui::prelude::*;
use crate::ui::store::Store;

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        sync::LazyLock,
    };

    use gettext::gettext;
    use gtk::{
        glib,
        glib::{ParamSpec, Value},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };

    use crate::{
        object::Rule,
        ui::{store::Store, widgets::FileChooserEntry},
    };

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
        pub store: OnceCell<Store>,
        pub original: OnceCell<Rule>,
        pub rule: RefCell<Rule>,
    }

    impl ScheduleRuleEditDialog {
        pub fn is_new(&self) -> bool {
            self.original.get().is_none()
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
            static PROPERTIES: LazyLock<Vec<ParamSpec>> = LazyLock::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<Store>("store")
                        .construct_only()
                        .build(),
                    glib::ParamSpecObject::builder::<Rule>("original")
                        .construct_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "store" => self.store.set(value.get().unwrap()).unwrap(),

                "original" => {
                    let maybe_rule: Option<Rule> = value.get().unwrap();
                    if let Some(rule) = maybe_rule {
                        self.rule.replace(rule.deep_clone());
                        self.original.set(rule).unwrap();
                        let rule = self.rule.borrow();
                        self.name_entry.set_text(&rule.name());
                        self.hourly_cell.set_value(rule.config().keep_hourly as f64);
                        self.daily_cell.set_value(rule.config().keep_daily as f64);
                        self.weekly_cell.set_value(rule.config().keep_weekly as f64);
                        self.monthly_cell
                            .set_value(rule.config().keep_monthly as f64);
                        self.yearly_cell.set_value(rule.config().keep_yearly as f64);
                    } else {
                        self.hourly_cell.set_value(24.0);
                        self.daily_cell.set_value(30.0);
                        self.monthly_cell.set_value(24.0);
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            if self.is_new() {
                self.save_button.set_label(&gettext("Create"));
                obj.set_title(Some(&gettext("New Rule")));
                self.remove_group.set_visible(false);
            } else {
                self.save_button.set_label(&gettext("Apply"));
                obj.set_title(Some(&gettext("Edit Rule")));
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
    pub fn new(store: &Store, rule: Option<&Rule>) -> Self {
        glib::Object::builder()
            .property("store", store)
            .property("original", rule)
            .build()
    }

    fn reload_subvolume_list(&self) {
        let imp = self.imp();
        if let Some(mut child) = imp.subvolume_list.first_child() {
            while let Some(sibling) = child.next_sibling() {
                imp.subvolume_list.remove(&child);
                child = sibling;
            }
        }
        for (idx, subvol) in imp.rule.borrow().config().subvolumes.iter().enumerate() {
            let remove_btn = gtk::Button::builder()
                .icon_name("list-remove-symbolic")
                .valign(gtk::Align::Center)
                .css_classes(vec!["flat".to_string(), "circular".to_string()])
                .build();
            remove_btn.connect_clicked(glib::clone!(@weak self as dialog => move |_| {
                dialog.imp().rule.borrow().config().subvolumes.remove(idx);
                dialog.reload_subvolume_list();
            }));
            let row = adw::ActionRow::builder()
                .title(subvol.path.to_string_lossy())
                .subtitle(subvol.target_dir.to_string_lossy())
                .build();
            row.add_prefix(&remove_btn);
            imp.subvolume_list.insert(&row, idx as i32);
        }
    }

    /// Return human readable error hint for invalid form.
    fn validate_form(&self) -> Option<&str> {
        let imp = self.imp();
        let name = imp.name_entry.text();
        if name.is_empty() {
            return Some("Name cannot be empty.");
        }
        if name.contains('/') {
            return Some("Name cannot contain '/'.");
        }
        return None;
    }

    #[template_callback]
    fn on_save_button_clicked(&self) {
        if let Some(err) = self.validate_form() {
            self.alert(err);
            return;
        }
        let imp = self.imp();
        let store = imp.store.get().unwrap();

        imp.rule.borrow().set_name(imp.name_entry.text());
        {
            let new_rule = imp.rule.borrow();
            let mut config = new_rule.config();
            config.keep_hourly = imp.hourly_cell.value() as u32;
            config.keep_daily = imp.daily_cell.value() as u32;
            config.keep_weekly = imp.weekly_cell.value() as u32;
            config.keep_monthly = imp.monthly_cell.value() as u32;
            config.keep_yearly = imp.yearly_cell.value() as u32;
        }

        let res = if let Some(original) = imp.original.get() {
            store.update_rule(original, &imp.rule.borrow())
        } else {
            store.create_rule(&imp.rule.borrow())
        };
        if let Err(e) = res {
            self.alert(&e.to_string());
        }
        self.close();
    }

    #[template_callback]
    fn on_remove_button_clicked(&self) {
        let imp = self.imp();
        if let Some(rule) = imp.original.get() {
            imp.store.get().unwrap().delete_rule(rule).unwrap();
        }
        self.close();
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
            imp.rule
                .borrow()
                .config()
                .subvolumes
                .push(RuleSubvolumeConfig {
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
