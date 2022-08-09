use adw::subclass::prelude::*;
use butter::show_error_dialog;
use gtk::{
    glib::{self, BoxedAnyObject, Object},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use crate::subvolume::SubvolumeManager;

use super::{ScheduleRuleEditDialog, ScheduleRuleRow};

mod imp {
    use glib::once_cell::sync::{Lazy, OnceCell};
    use gtk::glib::{ParamSpec, Value};

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/schedule_view.ui")]
    pub struct ScheduleView {
        #[template_child]
        pub rule_list: TemplateChild<gtk::ListBox>,
        pub client: OnceCell<SubvolumeManager>,
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
                    "client",
                    "client",
                    "client",
                    SubvolumeManager::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "client" => self
                    .client
                    .set(value.get::<SubvolumeManager>().unwrap())
                    .unwrap(),

                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.refresh();
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
    pub fn new(client: &SubvolumeManager) -> Self {
        Object::new(&[("client", client)]).expect("Failed to create ScheduleView")
    }

    #[template_callback]
    pub fn on_add_button_clicked(&self) {
        let win = self.root().and_then(|w| w.downcast::<gtk::Window>().ok());
        let dialog = ScheduleRuleEditDialog::new(self.imp().client.get().unwrap(), None);
        dialog.set_transient_for(win.as_ref());
        dialog.show();
    }

    pub fn refresh(&self) {
        match self.imp().client.get().unwrap().schedules() {
            Ok(model) => self.imp().rule_list.bind_model(Some(&model), |obj| {
                let boxed = obj.downcast_ref::<BoxedAnyObject>().unwrap();
                let ret = ScheduleRuleRow::new(boxed);
                ret.upcast()
            }),
            Err(err) => {
                let win = self.root().and_then(|w| w.downcast::<gtk::Window>().ok());
                show_error_dialog(
                    win.as_ref(),
                    &err.context("failed to reload schedule rules").to_string(),
                )
            }
        }
    }
}
