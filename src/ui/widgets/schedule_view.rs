use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::{object::Rule, ui::store::Store};

use super::{ScheduleRuleEditDialog, ScheduleRuleRow};

mod imp {
    use std::{cell::OnceCell, sync::LazyLock};

    use gtk::glib::{ParamSpec, Value};

    use crate::{object::Rule, ui::prelude::*};

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_view.ui")]
    pub struct ScheduleView {
        #[template_child]
        pub rule_list: TemplateChild<gtk::ListBox>,
        pub store: OnceCell<Store>,
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
            static PROPERTIES: LazyLock<Vec<ParamSpec>> = LazyLock::new(|| {
                vec![glib::ParamSpecObject::builder::<Store>("store")
                    .construct_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "store" => self.store.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            self.rule_list.bind_model(
                Some(self.store.get().unwrap().rule_model()),
                glib::clone!(
                    #[weak]
                    obj,
                    #[upgrade_or_panic]
                    move |schedule| {
                        let schedule = schedule.downcast_ref::<Rule>().unwrap();
                        let row = ScheduleRuleRow::new(schedule);
                        row.connect_activated(glib::clone!(
                            #[weak]
                            obj,
                            move |row| {
                                obj.show_edit_dialog(row.imp().rule.get());
                            }
                        ));
                        row.switch().connect_state_set(glib::clone!(
                            #[weak]
                            obj,
                            #[weak]
                            row,
                            #[upgrade_or]
                            glib::Propagation::Stop,
                            move |switch, state| {
                                let store = obj.imp().store.get().unwrap();
                                let rule = row.imp().rule.get().unwrap();
                                let new_rule = rule.deep_clone();
                                new_rule.set_is_enabled(state);
                                if let Err(error) = store.update_rule(&rule, &new_rule) {
                                    obj.alert(&error.to_string());
                                } else {
                                    switch.set_state(state);
                                }
                                glib::Propagation::Stop
                            }
                        ));

                        row.upcast()
                    }
                ),
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
    pub fn new(store: &Store) -> Self {
        glib::Object::builder().property("store", store).build()
    }

    pub fn show_edit_dialog(&self, schedule: Option<&Rule>) {
        let win = self.root().and_then(|w| w.downcast::<gtk::Window>().ok());
        let dialog = ScheduleRuleEditDialog::new(self.imp().store.get().unwrap(), schedule);
        dialog.set_transient_for(win.as_ref());
        dialog.set_visible(true);
    }

    #[template_callback]
    pub fn on_add_button_clicked(&self) {
        self.show_edit_dialog(None);
    }
}
