use adw::prelude::*;
use adw::{ActionRow, HeaderBar, ViewSwitcherTitle};
use gtk::gio;
use gtk::{Align, Button};

use crate::application::Application;
use crate::config;
use crate::window::Window;

pub fn build_ui(app: &Application) {
    let window = Window::new(&app);
    let view_stack = window.view_stack();
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

    let snapshot_list = window.snapshot_view().snapshot_list();

    for i in 0..5 {
        let r = ActionRow::builder()
            .title(format!("{} {}", "2022-01-17 22:00", i + 1).as_str())
            .subtitle("13 GB")
            .build();
        r.add_suffix(
            &Button::builder()
                .icon_name("edit-delete-symbolic")
                .valign(Align::Center)
                .css_classes(vec!["circular".into(), "flat".into()])
                .build(),
        );
        snapshot_list.append(&r);
    }

    window.content_box().prepend(&header_bar);
    window.present();

    let about_action = gio::SimpleAction::new("about", None);
    about_action.connect_activate(move |_, _| {
        let dialog = gtk::AboutDialog::builder()
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
