use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, ColumnViewColumn, MultiSelection, SignalListItemFactory};

use crate::{btrfs::snapshots, snapshot_object::SnapshotObject};

mod imp {
    use adw::subclass::prelude::*;
    use glib::once_cell::sync::OnceCell;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(CompositeTemplate, Default)]
    #[template(file = "../data/resources/ui/snapshot_view.ui")]
    pub struct SnapshotView {
        #[template_child(id = "snapshot_column_view")]
        pub column_view: TemplateChild<gtk::ColumnView>,
        pub model: OnceCell<gio::ListStore>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SnapshotView {
        const NAME: &'static str = "SnapshotView";
        type ParentType = adw::Bin;
        type Type = super::SnapshotView;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SnapshotView {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_models();
            obj.setup_column("path", "Path");
        }
    }
    impl WidgetImpl for SnapshotView {}
    impl BinImpl for SnapshotView {}
}

glib::wrapper! {
    pub struct SnapshotView(ObjectSubclass<imp::SnapshotView>)
        @extends gtk::Widget, adw::Bin;
}

impl SnapshotView {
    fn model(&self) -> &gio::ListStore {
        self.imp().model.get().expect("Failed to get model")
    }

    fn setup_models(&self) {
        let model = gio::ListStore::new(SnapshotObject::static_type());

        let snapshots = snapshots();
        for snapshot in snapshots {
            model.append(&SnapshotObject::from(snapshot));
        }

        let imp = self.imp();
        imp.model.set(model).expect("Failed to set model");
        let selection_model = MultiSelection::new(Some(self.model()));
        imp.column_view.set_model(Some(&selection_model));
    }

    fn setup_column(&self, property: &'static str, title: &str) {
        let column_view = self.imp().column_view.get();

        let factory = SignalListItemFactory::new();
        factory.connect_bind(|_, list_item| {
            let obj = list_item
                .item()
                .expect("Item must exist")
                .downcast::<SnapshotObject>()
                .expect("Item must be SnapshotObject");

            let lbl = gtk::Label::new(None);
            // TODO: cleanup
            let _binding = obj
                .bind_property(property, &lbl, "label")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            list_item.set_child(Some(&lbl));
        });
        let cvc = ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .expand(true)
            .build();
        column_view.append_column(&cvc);
    }
}
