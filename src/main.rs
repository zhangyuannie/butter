use adw::prelude::*;
use adw::{
    ActionRow, ApplicationWindow, CenteringPolicy, HeaderBar, ViewStack, ViewSwitcherBar,
    ViewSwitcherTitle,
};
use gtk::{
    Align, Application, Box, Button, Label, ListBox, MenuButton, Orientation, ScrolledWindow,
};

fn main() {
    let app = Application::builder()
        .application_id("com.github.zhangyuannie.buttercream")
        .build();

    app.connect_startup(|_| {
        adw::init();
    });

    app.connect_activate(build_ui);

    app.run();
}

fn build_header_bar(view_switcher_title: &ViewSwitcherTitle) -> HeaderBar {
    let header_bar = HeaderBar::builder()
        .centering_policy(CenteringPolicy::Strict)
        .build();

    let btn_add = Button::builder().icon_name("list-add-symbolic").build();
    let btn_menu = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .build();

    header_bar.pack_start(&btn_add);
    header_bar.pack_end(&btn_menu);

    header_bar.set_title_widget(Some(view_switcher_title));

    header_bar
}

fn build_ui(app: &Application) {
    let view_stack = ViewStack::builder().vexpand(true).build();
    let view_switcher_title = ViewSwitcherTitle::builder()
        .stack(&view_stack)
        .title("Buttercream")
        .build();

    let header_bar = build_header_bar(&view_switcher_title);

    let view_switcher_bar = ViewSwitcherBar::builder().stack(&view_stack).build();
    view_switcher_title
        .bind_property("title-visible", &view_switcher_bar, "reveal")
        .build();

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
                .css_classes(vec!["circular".to_string(), "flat".to_string()])
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
        .title("Buttercream")
        .content(&content)
        .build();
    window.present();
}
