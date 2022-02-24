use std::path::PathBuf;

use adw::subclass::prelude::*;
use gtk::{
    gdk,
    gio::{self, SimpleActionGroup},
    glib::{self, closure_local},
    prelude::*,
    BitsetIter, ColumnView, ColumnViewColumn, MultiSelection, SignalListItemFactory, Widget,
};

use crate::{
    rename_popover::RenamePopover, requester::daemon,
    snapshot_creation_window::SnapshotCreationWindow, snapshot_object::SnapshotObject,
};

mod imp {
    use adw::subclass::prelude::*;
    use glib::once_cell::sync::OnceCell;
    use gtk::{
        gio::{self, SimpleAction},
        glib,
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };
    use std::cell::RefCell;

    use crate::rename_popover::RenamePopover;

    #[derive(CompositeTemplate, Default)]
    #[template(file = "../data/resources/ui/snapshot_view.ui")]
    pub struct SnapshotView {
        #[template_child(id = "snapshot_column_view")]
        pub column_view: TemplateChild<gtk::ColumnView>,
        pub selection_menu: OnceCell<gtk::PopoverMenu>,
        pub rename_popover: RenamePopover,
        pub model: OnceCell<gio::ListStore>,
        pub single_select_actions: RefCell<Vec<SimpleAction>>,
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
            obj.setup_column("name", "Name", false);
            obj.setup_column("path", "Path", false);
            obj.setup_column("creation-time", "Created", false);
            obj.setup_column("parent-path", "Source", true);
            obj.setup_menu();
            obj.setup_clicks();
            obj.setup_rename_popover();
        }
        fn dispose(&self, obj: &Self::Type) {
            obj.teardown_rename_popover();
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

    fn set_single_select_actions_availability(&self, enable: bool) {
        for action in self.imp().single_select_actions.borrow().iter() {
            action.set_enabled(enable);
        }
    }

    fn setup_models(&self) {
        let model = gio::ListStore::new(SnapshotObject::static_type());
        let imp = self.imp();
        imp.model.set(model).expect("Failed to set model");
        self.refresh_model();
        let selection_model = MultiSelection::new(Some(self.model()));
        imp.column_view.set_model(Some(&selection_model));
    }

    fn refresh_model(&self) {
        let model = self.model();
        model.remove_all();
        let snapshots = daemon().snapshots();
        for snapshot in snapshots {
            model.append(&SnapshotObject::from(snapshot));
        }
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

    fn open_snapshot(&self, idx: u32) {
        let obj = self.model().item(idx).expect("Item must exist");
        let uri = format!("file://{}", obj.property::<String>("absolute-path"));
        println!("open_snapshot: show_uri: {}", uri);
        gtk::show_uri(None::<&gtk::Window>, uri.as_str(), gdk::CURRENT_TIME);
    }

    fn setup_menu(&self) {
        let imp = self.imp();
        let col_view = &imp.column_view.get();
        let actions = SimpleActionGroup::new();

        let open_action = gio::SimpleAction::new("open", None);
        open_action.connect_activate(glib::clone!(@weak self as view => move |_, _| {
            let selection_model = view.imp().column_view.get().model().unwrap();
            let selection = selection_model.selection();
            if selection.size() != 1 {
                println!("open: selection size should be 1");
            }
            let idx = selection.nth(0);
            view.open_snapshot(idx);
        }));

        let rename_action = gio::SimpleAction::new("rename", None);
        rename_action.connect_activate(glib::clone!(@weak self as view => move |_, _| {
            let imp = view.imp();
            let col_view = imp.column_view.get();
            let rename_popover = &imp.rename_popover;
            let selection_model = col_view.model().unwrap();
            let selection = selection_model.selection();
            if selection.size() != 1 {
                println!("rename: selection size should be 1");
            }
            let idx = selection.nth(0);

            let item = extract_ith_list_item(&col_view, idx).unwrap();
            rename_popover.set_target(idx);
            rename_popover.set_pointing_to(Some(&item.allocation()));
            rename_popover.popup();
        }));

        let delete_action = gio::SimpleAction::new("delete", None);
        delete_action.connect_activate(glib::clone!(@weak col_view, @weak self as view => move |_, _| {
            let selection_model = col_view.model().unwrap();
            let selection = selection_model.selection();
            for (_, idx) in BitsetIter::init_first(&selection) {
                let obj: SnapshotObject = selection_model
                    .item(idx)
                    .expect("Item must exist")
                    .downcast()
                    .unwrap();
                let absolute_path: String = obj.absolute_path();
                daemon().delete_snapshot(&absolute_path);
                println!("delete: {}", &absolute_path);
            }
            view.refresh_model();
        }));

        actions.add_action(&open_action);
        actions.add_action(&rename_action);
        actions.add_action(&delete_action);

        let mut single_actions = imp.single_select_actions.borrow_mut();
        single_actions.push(open_action);
        single_actions.push(rename_action);

        self.insert_action_group("view", Some(&actions));
    }

    fn setup_rename_popover(&self) {
        let imp = self.imp();
        let popover = &imp.rename_popover;
        let col_view = imp.column_view.get();
        let view = self.clone();
        popover.set_parent(&extract_column_list_view(&col_view));
        popover.connect_closure(
            "clicked",
            false,
            closure_local!(move |popover: RenamePopover, new_name: String| {
                let idx = popover.target();
                let obj: SnapshotObject = col_view
                    .model()
                    .unwrap()
                    .item(idx)
                    .expect("Item must exist")
                    .downcast()
                    .unwrap();
                let absolute_path: String = obj.absolute_path();

                let mut new_path = PathBuf::from(&absolute_path);
                new_path.set_file_name(new_name);

                daemon().rename_snapshot(absolute_path.as_str(), new_path.to_str().unwrap());

                view.refresh_model();

                popover.popdown();
            }),
        );
    }

    fn teardown_rename_popover(&self) {
        self.imp().rename_popover.unparent();
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
        col_view.connect_activate(glib::clone!(@weak self as view => move |_, idx| {
            view.open_snapshot(idx);
        }));

        // right click
        let gesture = gtk::GestureClick::builder()
            .button(gdk::BUTTON_SECONDARY)
            .build();
        gesture.connect_pressed(
            glib::clone!(@weak selection_menu, @weak self as view => move |gesture, _, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                let col_view: ColumnView = gesture.widget().downcast().unwrap();

                let header_rect = extract_header(&col_view).allocation();
                let clv_y = y - header_rect.y() as f64 - header_rect.height() as f64;

                if let Some(idx) = extract_row_from_column_list_view(&extract_column_list_view(&col_view), clv_y) {
                    let model = col_view.model().unwrap();
                    if !model.is_selected(idx) {
                        println!("gesture_pressed: select {} only", idx);
                        model.select_item(idx, true);
                    } else {
                        println!("gesture_pressed: already selected");
                    }

                    view.set_single_select_actions_availability(model.selection().size() <= 1);

                    let rect = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                    selection_menu.set_pointing_to(Some(&rect));
                    selection_menu.popup();
                }
            }),
        );
        selection_menu.set_parent(&col_view);
        col_view.add_controller(&gesture);
    }

    pub fn present_creation_window(&self) {
        let win = SnapshotCreationWindow::new();
        win.present();
    }
}

// TODO: hope there is a better way
fn extract_header(col_view: &ColumnView) -> Widget {
    let ret = col_view.first_child().unwrap();
    assert_eq!(ret.widget_name(), "GtkListItemWidget");
    ret
}
fn extract_column_list_view(col_view: &ColumnView) -> Widget {
    let ret = col_view.first_child().unwrap().next_sibling().unwrap();
    assert_eq!(ret.widget_name(), "GtkColumnListView");
    ret
}
fn extract_ith_list_item(col_view: &ColumnView, idx: u32) -> Option<Widget> {
    let list_view = extract_column_list_view(col_view);
    let mut cur = list_view.first_child()?;
    for _ in 0..idx {
        cur = cur.next_sibling()?;
    }
    return Some(cur);
}

/// y: relative to column_list_view, not column_view
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
