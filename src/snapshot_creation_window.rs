use std::path::PathBuf;

use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

use crate::subvolume::Subvolume;
use crate::subvolume_manager::SubvolumeManager;

mod imp {
    use glib::object::WeakRef;
    use glib::once_cell::sync::OnceCell;
    use gtk::glib::{once_cell::sync::Lazy, ParamFlags, ParamSpec, ParamSpecObject, Value};

    use crate::{file_chooser_entry::FileChooserEntry, subvolume_manager::SubvolumeManager};

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/snapshot_creation_window.ui")]
    pub struct SnapshotCreationWindow {
        #[template_child]
        pub create_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub location_entry: TemplateChild<FileChooserEntry>,
        #[template_child]
        pub subvol_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub readonly_switch: TemplateChild<gtk::Switch>,

        pub subvolume_manager: OnceCell<WeakRef<SubvolumeManager>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SnapshotCreationWindow {
        const NAME: &'static str = "SnapshotCreationWindow";
        type Type = super::SnapshotCreationWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SnapshotCreationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.create_button.set_sensitive(false);
            self.name_entry
                .connect_text_notify(glib::clone!(@weak obj => move |entry| {
                    obj.create_button().set_sensitive(entry.text_length() > 0);
                }));
            obj.setup_dropdown();
            self.location_entry.set_text("/var/snapshots");
            self.create_button.connect_clicked(glib::clone!(@weak obj => move |_| {
                let imp = obj.imp();
                let item = imp.subvol_dropdown.selected_item().unwrap().downcast::<Subvolume>().unwrap();
                obj.subvolume_manager().create_snapshot(
                    item.mounted_path().unwrap().as_str(),
                    obj.target_path().to_str().unwrap(),
                    imp.readonly_switch.is_active(),
                );
                obj.close();
            }));
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "subvolume-manager",
                    "subvolume-manager",
                    "subvolume-manager",
                    SubvolumeManager::static_type(),
                    ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "subvolume-manager" => self
                    .subvolume_manager
                    .set(value.get::<SubvolumeManager>().unwrap().downgrade())
                    .unwrap(),

                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SnapshotCreationWindow {}
    impl WindowImpl for SnapshotCreationWindow {}
}

glib::wrapper! {
    pub struct SnapshotCreationWindow(ObjectSubclass<imp::SnapshotCreationWindow>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SnapshotCreationWindow {
    pub fn new(subvolume_manager: &SubvolumeManager) -> Self {
        glib::Object::new(&[("subvolume-manager", subvolume_manager)]).unwrap()
    }

    fn setup_dropdown(&self) {
        let imp = self.imp();
        let filter = gtk::CustomFilter::new(|obj| {
            let subvol = obj.downcast_ref::<Subvolume>().unwrap();
            !subvol.is_snapshot() && subvol.mounted_path().is_some()
        });
        let model =
            gtk::FilterListModel::new(Some(self.subvolume_manager().model()), Some(&filter));

        let exp = gtk::ClosureExpression::new::<String, _, gtk::ClosureExpression>(
            None,
            glib::closure!(|sv: Subvolume| sv.name()),
        );

        imp.subvol_dropdown.set_expression(Some(&exp));
        imp.subvol_dropdown.set_model(Some(&model));
    }

    fn target_path(&self) -> PathBuf {
        let imp = self.imp();
        let mut ret = PathBuf::from(imp.location_entry.text().to_string());
        ret.push(imp.name_entry.text().to_string());
        ret
    }

    fn subvolume_manager(&self) -> SubvolumeManager {
        self.imp()
            .subvolume_manager
            .get()
            .unwrap()
            .upgrade()
            .unwrap()
    }

    fn create_button(&self) -> gtk::Button {
        self.imp().create_button.get()
    }
}
