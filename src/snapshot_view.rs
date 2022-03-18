use std::path::PathBuf;

use adw::subclass::prelude::*;
use gtk::{
    gdk,
    gio::{self, SimpleActionGroup},
    glib::{self, closure_local},
    prelude::*,
    BitsetIter, ColumnView, ColumnViewColumn, DialogFlags, MultiSelection, SignalListItemFactory,
    Widget,
};

use crate::{
    rename_popover::RenamePopover,
    snapshot_column_cell::SnapshotColumnCell,
    snapshot_creation_window::SnapshotCreationWindow,
    subvolume::{Subvolume, SubvolumeManager},
    window::Window,
};

mod imp {
    use adw::subclass::prelude::*;
    use glib::object::WeakRef;
    use glib::once_cell::sync::OnceCell;
    use gtk::{
        gio::SimpleAction,
        glib::{self, once_cell::sync::Lazy, ParamFlags, ParamSpec, ParamSpecObject, Value},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };
    use std::cell::RefCell;

    use crate::{rename_popover::RenamePopover, subvolume::SubvolumeManager};

    #[derive(CompositeTemplate, Default)]
    #[template(file = "../data/resources/ui/snapshot_view.ui")]
    pub struct SnapshotView {
        #[template_child(id = "snapshot_column_view")]
        pub column_view: TemplateChild<gtk::ColumnView>,
        pub selection_menu: OnceCell<gtk::PopoverMenu>,
        pub rename_popover: RenamePopover,
        pub model: OnceCell<gtk::FilterListModel>,
        pub single_select_actions: RefCell<Vec<SimpleAction>>,
        pub subvolume_manager: OnceCell<WeakRef<SubvolumeManager>>,
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
    impl WidgetImpl for SnapshotView {}
    impl BinImpl for SnapshotView {}
}

glib::wrapper! {
    pub struct SnapshotView(ObjectSubclass<imp::SnapshotView>)
        @extends gtk::Widget, adw::Bin;
}

impl SnapshotView {
    pub fn new(subvolume_manager: &SubvolumeManager) -> Self {
        glib::Object::new(&[("subvolume-manager", subvolume_manager)]).unwrap()
    }

    fn model(&self) -> &gtk::FilterListModel {
        self.imp().model.get().expect("Failed to get model")
    }

    fn set_single_select_actions_availability(&self, enable: bool) {
        for action in self.imp().single_select_actions.borrow().iter() {
            action.set_enabled(enable);
        }
    }

    fn setup_models(&self) {
        let imp = self.imp();
        let filter =
            gtk::CustomFilter::new(|obj| obj.downcast_ref::<Subvolume>().unwrap().is_snapshot());
        let model =
            gtk::FilterListModel::new(Some(self.subvolume_manager().model()), Some(&filter));
        imp.model.set(model).expect("Failed to set model");
        let selection_model = MultiSelection::new(Some(self.model()));
        imp.column_view.set_model(Some(&selection_model));
    }

    fn setup_column(&self, property: &'static str, title: &str, is_last: bool) {
        let column_view = self.imp().column_view.get();

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let cell = SnapshotColumnCell::new();
            list_item.set_child(Some(&cell));
        });
        factory.connect_bind(|_, list_item| {
            let obj = list_item
                .item()
                .expect("Item must exist")
                .downcast::<Subvolume>()
                .expect("Item must be Subvolume");

            let cell = list_item
                .child()
                .unwrap()
                .downcast::<SnapshotColumnCell>()
                .unwrap();

            let binding = obj
                .bind_property(property, &cell.label(), "label")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            cell.add_binding(binding);
        });
        factory.connect_unbind(move |_, list_item| {
            let cell = list_item
                .child()
                .expect("Child must exist")
                .downcast::<SnapshotColumnCell>()
                .expect("Child must be SnapshotColumnCell");

            cell.unbind_all();
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
        delete_action.connect_activate(
            glib::clone!(@weak col_view, @weak self as view => move |_, _| {
                let selection_model = col_view.model().unwrap();
                let selection = selection_model.selection();
                for (_, idx) in BitsetIter::init_first(&selection) {
                    let obj: Subvolume = selection_model
                        .item(idx)
                        .expect("Item must exist")
                        .downcast()
                        .unwrap();
                    view.subvolume_manager().delete_snapshot(&obj.mounted_path().unwrap());
                    println!("delete: {}", &obj.mounted_path().unwrap());
                }
            }),
        );

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
                let obj: Subvolume = col_view
                    .model()
                    .unwrap()
                    .item(idx)
                    .expect("Item must exist")
                    .downcast()
                    .unwrap();

                let mut new_path = PathBuf::from(&obj.mounted_path().unwrap());
                new_path.set_file_name(new_name);

                let res = view.subvolume_manager().rename_snapshot(
                    obj.mounted_path().unwrap().as_str(),
                    new_path.to_str().unwrap(),
                );

                if let Err(s) = res {
                    let win = view.root().unwrap().downcast::<Window>().unwrap();
                    let dialog = gtk::MessageDialog::new(
                        Some(&win),
                        DialogFlags::DESTROY_WITH_PARENT | DialogFlags::MODAL,
                        gtk::MessageType::Error,
                        gtk::ButtonsType::Close,
                        &s,
                    );
                    dialog.connect_response(|dialog, _| {
                        dialog.destroy();
                    });
                    dialog.show();
                }
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
        let win = SnapshotCreationWindow::new(&self.subvolume_manager());
        win.present();
    }

    fn subvolume_manager(&self) -> SubvolumeManager {
        self.imp()
            .subvolume_manager
            .get()
            .unwrap()
            .upgrade()
            .unwrap()
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
