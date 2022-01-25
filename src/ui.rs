use adw::prelude::*;
use adw::{ActionRow, ApplicationWindow, HeaderBar, ViewStack, ViewSwitcherBar, ViewSwitcherTitle};
use gtk::gio;
use gtk::{Align, Box, Button, Label, ListBox, Orientation, ScrolledWindow};

use crate::application::Application;
use crate::config;

pub fn build_ui(app: &Application) {
    let view_stack = ViewStack::builder().vexpand(true).build();
    let view_switcher_title = ViewSwitcherTitle::builder()
        .stack(&view_stack)
        .title(config::APP_NAME)
        .build();

    let view_switcher_bar = ViewSwitcherBar::builder().stack(&view_stack).build();
    view_switcher_title
        .bind_property("title-visible", &view_switcher_bar, "reveal")
        .build();

    let header_bar_builder =
        gtk::Builder::from_string(include_str!("../data/resources/ui/header_bar.ui"));

    let header_bar: HeaderBar = header_bar_builder.object("header_bar").unwrap();
    header_bar.set_title_widget(Some(&view_switcher_title));

    let snapshot_list = ListBox::builder().build();

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

    let snapshot_page = view_stack.add(&ScrolledWindow::builder().child(&snapshot_list).build());
    snapshot_page.set_name(Some("snapshot"));
    snapshot_page.set_title(Some("Snapshot"));
    snapshot_page.set_icon_name(Some("insert-object-symbolic"));

    let schedule_page = view_stack.add(&Label::new(Some("test")));
    schedule_page.set_name(Some("schedule"));
    schedule_page.set_title(Some("Schedule"));
    schedule_page.set_icon_name(Some("alarm-symbolic"));

    let content = Box::builder().orientation(Orientation::Vertical).build();
    content.append(&header_bar);
    content.append(&view_stack);
    content.append(&view_switcher_bar);

    let window = ApplicationWindow::builder()
        .application(app)
        .title(config::APP_NAME)
        .content(&content)
        .build();
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
