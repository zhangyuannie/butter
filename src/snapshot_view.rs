use adw::subclass::prelude::*;
use gtk::{
    gdk, gio, glib, prelude::*, ColumnView, ColumnViewColumn, MultiSelection,
    SignalListItemFactory, Widget,
};

use crate::{requester::daemon, snapshot_object::SnapshotObject};

mod imp {
    use adw::subclass::prelude::*;
    use glib::once_cell::sync::OnceCell;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(CompositeTemplate, Default)]
    #[template(file = "../data/resources/ui/snapshot_view.ui")]
    pub struct SnapshotView {
        #[template_child(id = "snapshot_column_view")]
        pub column_view: TemplateChild<gtk::ColumnView>,
        pub selection_menu: OnceCell<gtk::PopoverMenu>,
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
            obj.setup_column("path", "Path", false);
            obj.setup_column("creation-time", "Created", false);
            obj.setup_column("parent-path", "Source", true);
            obj.setup_menu();
            obj.setup_clicks();
        }
        fn dispose(&self, obj: &Self::Type) {
            if let Some(widget) = obj.imp().selection_menu.get() {
                widget.unparent()
            }
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

        let snapshots = daemon().snapshots();
        for snapshot in snapshots {
            model.append(&SnapshotObject::from(snapshot));
        }

        let imp = self.imp();
        imp.model.set(model).expect("Failed to set model");
        let selection_model = MultiSelection::new(Some(self.model()));
        imp.column_view.set_model(Some(&selection_model));
    }

    fn setup_column(&self, property: &'static str, title: &str, is_last: bool) {
        let column_view = self.imp().column_view.get();

        let factory = SignalListItemFactory::new();
        factory.connect_bind(|_, list_item| {
            let obj = list_item
                .item()
                .expect("Item must exist")
                .downcast::<SnapshotObject>()
                .expect("Item must be SnapshotObject");

            let lbl = gtk::Label::new(None);
            let binding = obj
                .bind_property(property, &lbl, "label")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            obj.imp().binding.borrow_mut().replace(binding);

            list_item.set_child(Some(&lbl));
        });
        factory.connect_unbind(move |_, list_item| {
            let obj = list_item
                .item()
                .expect("Item must exist")
                .downcast::<SnapshotObject>()
                .expect("Item must be SnapshotObject");

            obj.imp().binding.borrow().as_ref().unwrap().unbind();
        });
        let cvc = ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .expand(is_last)
            .resizable(!is_last)
            .build();
        column_view.append_column(&cvc);
    }

    fn setup_menu(&self) {
        let open_action = gio::SimpleAction::new("open", None);
        // self.insert_action_group(name, group)
        let rename_action = gio::SimpleAction::new("rename", None);
        let delete_action = gio::SimpleAction::new("delete", None);

    }

    fn setup_clicks(&self) {
        let imp = self.imp();
        let col_view = imp.column_view.get();

        let selection_menu_builder =
            gtk::Builder::from_string(include_str!("../data/resources/ui/selection_menu.ui"));
        imp.selection_menu
            .set(selection_menu_builder.object("selection_menu").unwrap())
            .unwrap();
        let selection_menu = imp.selection_menu.get().unwrap();

        // double click
        col_view.connect_activate(|cv, idx| {
            let model = cv.model().unwrap();
            let obj = model.item(idx).expect("Item must exist");

            gtk::show_uri(
                None::<&gtk::Window>,
                format!("file:///{}", obj.property::<String>("path")).as_str(),
                gdk::CURRENT_TIME,
            );
        });

        // right click
        let gesture = gtk::GestureClick::builder()
            .button(gdk::BUTTON_SECONDARY)
            .build();
        gesture.connect_pressed(
            glib::clone!(@weak selection_menu => move |gesture, _, x, y| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    let col_list_view = gesture.widget();
                    assert_eq!(col_list_view.widget_name(), "GtkColumnListView");
                    
                    if let Some(idx) = extract_row_from_column_list_view(&col_list_view, y) {
                        let col_view = col_list_view.parent().unwrap().downcast::<ColumnView>().unwrap();
                        let model = col_view.model().unwrap();
                        if !model.is_selected(idx) {
                            model.select_item(idx, true);
                        }
                        
                        let rect = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                        selection_menu.set_pointing_to(Some(&rect));
                        selection_menu.popup();
                        println!("{}", idx);
                    }
            }),
        );
        let clv = extract_column_list_view(&col_view);
        selection_menu.set_parent(&clv);
        clv.add_controller(&gesture);
    }
}

// TODO: hope there is a better way
fn extract_column_list_view(col_view: &ColumnView) -> Widget {
    let col_list_view = col_view.first_child().unwrap().next_sibling().unwrap();
    assert_eq!(col_list_view.widget_name(), "GtkColumnListView");
    col_list_view
}

fn extract_row_from_column_list_view(column_list_view: &Widget, y: f64) -> Option<u32> {
    let mut cur = column_list_view.first_child()?;
    let mut idx = 0;

    loop {
        assert_eq!(cur.widget_name(), "GtkListItemWidget");
        let rect = cur.allocation();
        if rect.y() as f64 <= y && y < (rect.y() + rect.height()) as f64 {
            return Some(idx);
        }
        idx += 1;
        cur = cur.next_sibling()?;
    }
}
