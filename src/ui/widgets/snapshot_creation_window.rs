use std::path::PathBuf;

use gtk::subclass::prelude::*;
use gtk::{glib, prelude::*, CompositeTemplate};

use super::FileChooserEntry;
use crate::subvolume::GSubvolume;
use crate::ui::store::Store;
mod imp {
    use glib::object::WeakRef;
    use glib::once_cell::sync::OnceCell;
    use gtk::glib::{once_cell::sync::Lazy, ParamFlags, ParamSpec, ParamSpecObject, Value};

    use crate::ui::{prelude::*, store::Store};

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

        pub store: OnceCell<WeakRef<Store>>,
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
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            self.create_button.set_sensitive(false);
            self.name_entry
                .connect_text_notify(glib::clone!(@weak obj => move |entry| {
                    obj.create_button().set_sensitive(entry.text_length() > 0);
                }));
            obj.setup_dropdown();
            self.location_entry.set_text("/var/snapshots");
            self.create_button.connect_clicked(glib::clone!(@weak obj => move |_| {
                let imp = obj.imp();
                let item = imp.subvol_dropdown.selected_item().unwrap().downcast::<GSubvolume>().unwrap();
                let res = obj.store().create_snapshot(
                    item.mount_path().unwrap(),
                    obj.target_path().as_path(),
                    imp.readonly_switch.is_active(),
                );

                match res {
                    Ok(_) => obj.close(),
                    Err(error) => obj.alert(&error.to_string()),
                }
            }));
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "store",
                    None,
                    None,
                    Store::static_type(),
                    ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "store" => self
                    .store
                    .set(value.get::<Store>().unwrap().downgrade())
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
    pub fn new(store: &Store) -> Self {
        glib::Object::builder().property("store", store).build()
    }

    fn setup_dropdown(&self) {
        let imp = self.imp();
        let filter = gtk::CustomFilter::new(|obj| {
            let subvol = obj.downcast_ref::<GSubvolume>().unwrap();
            subvol.is_protected()
        });
        let model = gtk::FilterListModel::new(Some(self.store().model()), Some(filter));

        let exp = gtk::ClosureExpression::new::<String>(
            &[] as &[gtk::Expression],
            glib::closure!(|sv: GSubvolume| {
                let path = String::from(sv.subvol_path().to_string_lossy());
                if path == "/" {
                    "<FS_TREE>".to_string()
                } else {
                    path
                }
            }),
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

    fn store(&self) -> Store {
        self.imp().store.get().unwrap().upgrade().unwrap()
    }

    fn create_button(&self) -> gtk::Button {
        self.imp().create_button.get()
    }
}
