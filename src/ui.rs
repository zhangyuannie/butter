use adw::prelude::*;
use adw::{HeaderBar, ViewSwitcherTitle};
use gettext::gettext;
use gtk::{gio, glib, Label};

use crate::application::Application;
use crate::config;
use crate::snapshot_view::SnapshotView;
use crate::subvolume::GBtrfsFilesystem;
use crate::window::Window;

pub fn build_ui(app: &Application) {
    let window = Window::new(&app);
    let view_stack = window.view_stack();

    let snapshot_page = view_stack.add(&SnapshotView::new(&app.subvolume_manager()));
    snapshot_page.set_name(Some("snapshot"));
    snapshot_page.set_title(Some(gettext("Snapshot").as_str()));
    snapshot_page.set_icon_name(Some("insert-object-symbolic"));

    let schedule_page = view_stack.add(&Label::new(Some("Placeholder")));
    schedule_page.set_name(Some("schedule"));
    schedule_page.set_title(Some(gettext("Schedule").as_str()));
    schedule_page.set_icon_name(Some("alarm-symbolic"));

    let view_switcher_title = ViewSwitcherTitle::builder()
        .stack(&view_stack)
        .title(config::APP_NAME)
        .build();

    view_switcher_title
        .bind_property("title-visible", &window.switcher_bar(), "reveal")
        .build();

    let header_bar_builder =
        gtk::Builder::from_string(include_str!("../data/resources/ui/header_bar.ui"));

    let header_bar: HeaderBar = header_bar_builder.object("header_bar").unwrap();
    header_bar.set_title_widget(Some(&view_switcher_title));

    // filesystem dropdown
    {
        let exp = gtk::ClosureExpression::new::<String, _, gtk::ClosureExpression>(
            None,
            glib::closure!(|sv: GBtrfsFilesystem| sv.display()),
        );
        let fs_dropdown =
            gtk::DropDown::new(Some(app.subvolume_manager().filesystems()), Some(&exp));

        header_bar.pack_end(&fs_dropdown);
    }

    window.content_box().prepend(&header_bar);
    window.present();

    let about_action = gio::SimpleAction::new("about", None);
    about_action.connect_activate(move |_, _| {
        let dialog = gtk::AboutDialog::builder()
            .logo_icon_name(config::APP_ID)
            .copyright("© 2022 Zhangyuan Nie")
            .license_type(gtk::License::Gpl30Only)
            .program_name(config::APP_NAME)
            .authors(vec!["Zhangyuan Nie".into()])
            .transient_for(&window)
            .modal(true)
            .version(config::APP_VERSION)
            .build();

        dialog.show();
    });
    app.add_action(&about_action);
}
