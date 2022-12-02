use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{
    glib::{self, Object},
    CompositeTemplate,
};

use crate::schedule_repo::{ScheduleObject, ScheduleRepo};

use super::{ScheduleRuleEditDialog, ScheduleRuleRow};

mod imp {
    use glib::once_cell::sync::{Lazy, OnceCell};
    use gtk::glib::{ParamSpec, Value};

    use crate::schedule_repo::ScheduleObject;
    use crate::ui::show_error_dialog;

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_view.ui")]
    pub struct ScheduleView {
        #[template_child]
        pub rule_list: TemplateChild<gtk::ListBox>,
        pub repo: OnceCell<ScheduleRepo>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScheduleView {
        const NAME: &'static str = "ScheduleView";
        type ParentType = adw::Bin;
        type Type = super::ScheduleView;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScheduleView {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "repo",
                    "repo",
                    "repo",
                    ScheduleRepo::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "repo" => self.repo.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.instance();
            self.rule_list.bind_model(
                Some(self.repo.get().unwrap().model()),
                glib::clone!(@weak obj => @default-panic, move |schedule| {
                    let schedule = schedule.downcast_ref::<ScheduleObject>().unwrap();
                    let row = ScheduleRuleRow::new(schedule);
                    row.connect_activated(glib::clone!(@weak obj => move |row| {
                        obj.show_edit_dialog(row.imp().rule.get());
                    }));
                    row.switch().connect_state_set(glib::clone!(@weak obj, @weak row => @default-return glib::signal::Inhibit(true), move |switch, state| {
                        let repo = obj.imp().repo.get().unwrap();
                        let rule = row.imp().rule.get().unwrap().clone();
                        rule.borrow_mut().data.is_enabled = state;
                        if let Err(error) = repo.persist(&rule) {
                            let win = obj.root().and_then(|w| w.downcast::<gtk::Window>().ok());
                            show_error_dialog(win.as_ref(), &error.to_string());
                        } else {
                            switch.set_state(state);
                        }
                        glib::signal::Inhibit(true)
                    }));

                    row.upcast()
                }),
            );
        }
    }
    impl WidgetImpl for ScheduleView {}
    impl BinImpl for ScheduleView {}
}

glib::wrapper! {
    pub struct ScheduleView(ObjectSubclass<imp::ScheduleView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl ScheduleView {
    pub fn new(repo: &ScheduleRepo) -> Self {
        Object::new(&[("repo", repo)])
    }

    pub fn show_edit_dialog(&self, schedule: Option<&ScheduleObject>) {
        let win = self.root().and_then(|w| w.downcast::<gtk::Window>().ok());
        let dialog = ScheduleRuleEditDialog::new(self.imp().repo.get().unwrap(), schedule);
        dialog.set_transient_for(win.as_ref());
        dialog.show();
    }

    #[template_callback]
    pub fn on_add_button_clicked(&self) {
        self.show_edit_dialog(None);
    }
}
