use gtk::{glib, subclass::prelude::*};

mod imp {
    use adw::subclass::prelude::*;
    use gtk::glib::{self, once_cell::sync::Lazy};
    use gtk::{prelude::*, CompositeTemplate};

    #[allow(non_snake_case, non_upper_case_globals)]
    mod Property {
        pub const TitleStart: usize = 1;
        pub const TitleEnd: usize = 2;
    }

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/zhangyuannie/butter/ui/app_header_bar.ui")]
    pub struct AppHeaderBar {
        #[template_child]
        pub view_switcher_title: TemplateChild<adw::ViewSwitcherTitle>,
        #[template_child]
        pub fs_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub start_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub end_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub switch: TemplateChild<gtk::Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppHeaderBar {
        const NAME: &'static str = "AppHeaderBar";
        type Type = super::AppHeaderBar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AppHeaderBar {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                let ret = vec![
                    glib::ParamSpecString::new(
                        "title-start",
                        "title-start",
                        "title-start",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "title-end",
                        "title-end",
                        "title-end",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                ];
                assert_eq!(ret[Property::TitleStart - 1].name(), "title-start");
                assert_eq!(ret[Property::TitleEnd - 1].name(), "title-end");
                ret
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            match id {
                Property::TitleStart => self.start_stack.visible_child_name().to_value(),
                Property::TitleEnd => self.end_stack.visible_child_name().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, id: usize, value: &glib::Value, _pspec: &glib::ParamSpec) {
            match id {
                Property::TitleStart => self
                    .start_stack
                    .set_visible_child_name(value.get().unwrap()),
                Property::TitleEnd => self.end_stack.set_visible_child_name(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for AppHeaderBar {}
    impl BinImpl for AppHeaderBar {}
}

glib::wrapper! {
    pub struct AppHeaderBar(ObjectSubclass<imp::AppHeaderBar>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for AppHeaderBar {
    fn default() -> Self {
        Self::new()
    }
}

impl AppHeaderBar {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn view_switcher_title(&self) -> &adw::ViewSwitcherTitle {
        self.imp().view_switcher_title.as_ref()
    }

    pub fn fs_dropdown(&self) -> &gtk::DropDown {
        self.imp().fs_dropdown.as_ref()
    }

    pub fn switch(&self) -> &gtk::Switch {
        self.imp().switch.as_ref()
    }
}
