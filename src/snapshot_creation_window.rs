use std::path::PathBuf;

use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, CompositeTemplate};

use crate::requester::daemon;
use crate::subvolume::Subvolume;

mod imp {
    use crate::file_chooser_entry::FileChooserEntry;
    use crate::requester::daemon;
    use crate::subvolume::Subvolume;

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
            obj.setup_dropdown();
            self.location_entry.set_text("/var/snapshots");
            self.create_button.connect_clicked(glib::clone!(@weak obj => move |_| {
                let imp = obj.imp();
                let item = imp.subvol_dropdown.selected_item().unwrap().downcast::<Subvolume>().unwrap();
                daemon().create_snapshot(item.mounted_path().as_str(), obj.target_path().to_str().unwrap());
            }));
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
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create SnapshotCreationWindow")
    }

    fn setup_dropdown(&self) {
        let subvols = daemon().subvolumes();
        let model = gio::ListStore::new(Subvolume::static_type());
        for subvol in subvols {
            if subvol.snapshot_source_path.is_none() {
                model.append(&Subvolume::from(subvol));
            }
        }
        let exp = gtk::ClosureExpression::new::<String, _, gtk::ClosureExpression>(
            None,
            glib::closure!(|sv: Subvolume| sv.name()),
        );
        let imp = self.imp();
        imp.subvol_dropdown.set_expression(Some(&exp));
        imp.subvol_dropdown.set_model(Some(&model));
    }

    fn target_path(&self) -> PathBuf {
        let imp = self.imp();
        let mut ret = PathBuf::from(imp.location_entry.text().to_string());
        ret.push(imp.name_entry.text().to_string());
        ret
    }
}

impl Default for SnapshotCreationWindow {
    fn default() -> Self {
        Self::new()
    }
}
