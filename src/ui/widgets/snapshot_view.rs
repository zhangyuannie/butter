use adw::subclass::prelude::*;
use gtk::{
    gdk, gio, glib, prelude::*, BitsetIter, ColumnView, ColumnViewColumn, SignalListItemFactory,
    Widget,
};

use crate::{
    subvolume::{Attribute, GSubvolume},
    ui::{
        show_error_dialog,
        store::Store,
        widgets::{AppWindow, SnapshotCreationWindow, SubvolumeLabelCell},
    },
};

mod imp {
    use adw::subclass::prelude::*;
    use gettext::gettext;
    use glib::object::WeakRef;
    use glib::once_cell::sync::OnceCell;
    use gtk::{
        gdk, gio,
        gio::SimpleAction,
        glib::{self, once_cell::sync::Lazy, ParamFlags, ParamSpec, ParamSpecObject, Value},
        prelude::*,
        CompositeTemplate,
    };
    use std::cell::RefCell;

    use crate::{
        subvolume::{Attribute, GSubvolume},
        ui::{store::Store, widgets::SnapshotRenamePopover},
    };

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/zhangyuannie/butter/ui/snapshot_view.ui")]
    pub struct SnapshotView {
        #[template_child(id = "snapshot_column_view")]
        pub column_view: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub header_menu_model: TemplateChild<gio::MenuModel>,
        #[template_child]
        pub selection_menu: TemplateChild<gtk::PopoverMenu>,
        pub rename_popover: SnapshotRenamePopover,
        pub single_select_actions: RefCell<Vec<SimpleAction>>,
        pub store: OnceCell<WeakRef<Store>>,
        pub actions: gio::SimpleActionGroup,
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
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_model();
            let obj = self.instance();

            let header_menu = self.header_menu_model.get();

            obj.setup_menu();

            obj.setup_column(
                Attribute::Name,
                gettext("Name").as_str(),
                false,
                &header_menu,
            );
            obj.setup_column(
                Attribute::Path,
                gettext("Path").as_str(),
                false,
                &header_menu,
            );
            let created_col = obj.setup_column(
                Attribute::Created,
                gettext("Created").as_str(),
                false,
                &header_menu,
            );
            obj.setup_column(
                Attribute::ParentPath,
                gettext("Source").as_str(),
                true,
                &header_menu,
            );
            // set default sort order
            self.column_view
                .sort_by_column(Some(&created_col), gtk::SortType::Descending);

            obj.setup_clicks();
            obj.setup_rename_popover();
        }

        fn dispose(&self) {
            self.instance().teardown_rename_popover();
            self.selection_menu.unparent();
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
    impl WidgetImpl for SnapshotView {}
    impl BinImpl for SnapshotView {}

    impl SnapshotView {
        fn setup_model(&self) {
            let filter = gtk::CustomFilter::new(|obj| {
                obj.downcast_ref::<GSubvolume>().unwrap().is_snapshot()
            });
            let model = gtk::FilterListModel::new(Some(self.store().model()), Some(&filter));

            let model = gtk::SortListModel::new(Some(&model), self.column_view.sorter().as_ref());
            let model = gtk::MultiSelection::new(Some(&model));
            self.column_view.set_model(Some(&model));
        }

        pub fn show_rename_popover(&self, pointing_to: &gdk::Rectangle, prefill: &str) {
            self.rename_popover.set_text(prefill);
            self.rename_popover.set_pointing_to(Some(pointing_to));
            self.rename_popover.popup();
        }

        pub fn store(&self) -> Store {
            self.store.get().unwrap().upgrade().unwrap()
        }
    }
}

glib::wrapper! {
    pub struct SnapshotView(ObjectSubclass<imp::SnapshotView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SnapshotView {
    pub fn new(store: &Store) -> Self {
        glib::Object::new(&[("store", store)])
    }

    fn model(&self) -> gtk::SelectionModel {
        self.imp().column_view.model().expect("Failed to get model")
    }

    fn set_single_select_actions_availability(&self, enable: bool) {
        for action in self.imp().single_select_actions.borrow().iter() {
            action.set_enabled(enable);
        }
    }

    fn setup_column(
        &self,
        attribute: Attribute,
        title: &str,
        expand: bool,
        menu: &gio::MenuModel,
    ) -> ColumnViewColumn {
        let column_view = self.imp().column_view.get();

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let cell = SubvolumeLabelCell::new();
            item.set_child(Some(&cell));
        });
        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let obj: GSubvolume = item.item().unwrap().downcast().unwrap();
            let cell: SubvolumeLabelCell = item.child().unwrap().downcast().unwrap();
            cell.label().set_label(&obj.attribute_str(attribute));
        });
        factory.connect_unbind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let cell = item
                .child()
                .unwrap()
                .downcast::<SubvolumeLabelCell>()
                .unwrap();
            cell.label().set_label("");
        });
        let cvc = ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .expand(expand)
            .resizable(true)
            .header_menu(menu)
            .sorter(&attribute.sorter())
            .build();

        let show_action = gio::PropertyAction::new(
            format!("show-{}", attribute.as_str()).as_str(),
            &cvc,
            "visible",
        );

        self.imp().actions.add_action(&show_action);

        column_view.append_column(&cvc);
        cvc
    }

    fn open_snapshot(&self, idx: u32) {
        let obj = self.model().item(idx).expect("Item must exist");
        let uri = format!("file://{}", obj.property::<String>("path"));
        println!("open_snapshot: show_uri: {}", uri);
        gtk::show_uri(None::<&gtk::Window>, uri.as_str(), gdk::CURRENT_TIME);
    }

    fn setup_menu(&self) {
        let imp = self.imp();
        let col_view = &imp.column_view.get();

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
            let selection_model = col_view.model().unwrap();
            let selection = selection_model.selection();
            if selection.size() != 1 {
                println!("rename: selection size should be 1");
            }
            let idx = selection.nth(0);
            let item = extract_ith_list_item(&col_view, idx).unwrap();
            let subvol: GSubvolume = selection_model.item(idx).unwrap().downcast().unwrap();
            imp.show_rename_popover(&item.allocation(), &subvol.name());
        }));

        let delete_action = gio::SimpleAction::new("delete", None);
        delete_action.connect_activate(
            glib::clone!(@weak col_view, @weak self as view => move |_, _| {
                let selection_model = col_view.model().unwrap();
                let selection = selection_model.selection();
                for (_, idx) in BitsetIter::init_first(&selection) {
                    let obj: GSubvolume = selection_model
                        .item(idx)
                        .expect("Item must exist")
                        .downcast()
                        .unwrap();
                    view.store().delete_snapshot(obj.mount_path().unwrap()).unwrap();
                    println!("delete: {}", obj.mount_path().unwrap().display());
                }
            }),
        );

        let actions = &imp.actions;
        actions.add_action(&open_action);
        actions.add_action(&rename_action);
        actions.add_action(&delete_action);

        let mut single_actions = imp.single_select_actions.borrow_mut();
        single_actions.push(open_action);
        single_actions.push(rename_action);
        self.insert_action_group("view", Some(actions));
    }

    fn setup_rename_popover(&self) {
        let imp = self.imp();
        let popover = &imp.rename_popover;
        let col_view = imp.column_view.get();
        popover.set_parent(&extract_column_list_view(&col_view));
        popover.connect_clicked(glib::clone!(@weak self as view => move |popover| {
            let selection_model = view.imp().column_view.model().unwrap();
            let selection = selection_model.selection();
            if selection.size() != 1 {
                println!("rename: selection size should be 1");
                return;
            }
            let obj: GSubvolume = selection_model
                .item(selection.nth(0))
                .expect("Item must exist")
                .downcast()
                .unwrap();

            let mut new_path = obj.mount_path().unwrap().to_path_buf();
            new_path.set_file_name(popover.text());

            popover.popdown();

            let res = view
                .store()
                .rename_snapshot(obj.mount_path().unwrap(), new_path.as_path());

            if let Err(error) = res {
                let win = view.root().unwrap().downcast::<AppWindow>().unwrap();
                show_error_dialog(Some(&win), &error.to_string());
            }
        }));
    }

    fn teardown_rename_popover(&self) {
        self.imp().rename_popover.unparent();
    }

    fn setup_clicks(&self) {
        let imp = self.imp();
        let col_view = imp.column_view.get();

        let selection_menu = imp.selection_menu.get();

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
                let col_view: ColumnView = gesture.widget().downcast().unwrap();

                let header_rect = extract_header(&col_view).allocation();
                let clv_y = y - header_rect.y() as f64 - header_rect.height() as f64;

                if let Some(idx) = extract_row_from_column_list_view(&extract_column_list_view(&col_view), clv_y) {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
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
        let win = SnapshotCreationWindow::new(&self.store());
        let app_win = self.root().and_then(|w| w.downcast::<gtk::Window>().ok());
        win.set_transient_for(app_win.as_ref());
        win.set_modal(true);
        win.present();
    }

    fn store(&self) -> Store {
        self.imp().store()
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
        if cur.widget_name() == "GtkListItemWidget" {
            let rect = cur.allocation();
            if rect.y() as f64 <= y && y < (rect.y() + rect.height()) as f64 {
                return Some(idx);
            }
        }
        idx += 1;
        cur = cur.next_sibling()?;
    }
}
