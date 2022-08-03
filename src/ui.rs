use adw::prelude::*;
use gettext::gettext;
use gtk::{gio, glib};

use crate::application::Application;
use crate::config;
use crate::snapshot_view::SnapshotView;
use crate::subvolume::GBtrfsFilesystem;
use crate::widgets::ScheduleView;
use crate::window::Window;

pub fn build_ui(app: &Application) {
    let window = Window::new(&app);
    let view_stack = window.view_stack();
    let header_bar = window.header_bar();

    let snapshot_page = view_stack.add(&SnapshotView::new(&app.subvolume_manager()));
    snapshot_page.set_name(Some("snapshot"));
    snapshot_page.set_title(Some(gettext("Snapshot").as_str()));
    snapshot_page.set_icon_name(Some("insert-object-symbolic"));

    let schedule_page = view_stack.add(&ScheduleView::new(&app.subvolume_manager()));
    schedule_page.set_name(Some("schedule"));
    schedule_page.set_title(Some(gettext("Schedule").as_str()));
    schedule_page.set_icon_name(Some("alarm-symbolic"));

    let view_switcher_title = header_bar.view_switcher_title();
    view_switcher_title.set_stack(Some(&view_stack));
    view_switcher_title
        .bind_property("title-visible", &window.switcher_bar(), "reveal")
        .build();

    // filesystem dropdown
    {
        let exp = gtk::ClosureExpression::new::<String, _, gtk::ClosureExpression>(
            None,
            glib::closure!(|sv: GBtrfsFilesystem| sv.display()),
        );

        let fs_dropdown = header_bar.fs_dropdown();
        fs_dropdown.set_expression(Some(&exp));
        fs_dropdown.set_model(Some(app.subvolume_manager().filesystems()));

        let app = app.clone();
        fs_dropdown.connect_selected_notify(move |dd| {
            if let Some(fs) = dd.selected_item() {
                let fs: GBtrfsFilesystem = fs.downcast().expect("Object must be GBtrfsFilesystem");
                let fs = fs.data().clone();
                app.subvolume_manager().set_filesystem(fs).unwrap();
            }
        });
    }
    {
        let subvol_mgr = app.subvolume_manager();

        let switch = header_bar.switch();
        switch.set_state(app.subvolume_manager().is_schedule_enabled());
        switch.connect_state_set(glib::clone!(@weak subvol_mgr => @default-return glib::signal::Inhibit(true), move |switch, state| {
            if let Err(error) = subvol_mgr.set_is_schedule_enabled(state) {
                let dialog = gtk::MessageDialog::new(
                    None::<&gtk::Window>,
                    gtk::DialogFlags::DESTROY_WITH_PARENT |  gtk::DialogFlags::MODAL,
                    gtk::MessageType::Error,
                    gtk::ButtonsType::Close,
                    &error.to_string(),
                );
                dialog.connect_response(|dialog, _| {
                    dialog.destroy();
                });
                dialog.show();
            }
            println!("e: {}", subvol_mgr.is_schedule_enabled());
            switch.set_state(subvol_mgr.is_schedule_enabled());
            glib::signal::Inhibit(true)
        }));
    }

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
