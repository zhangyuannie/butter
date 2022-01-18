use adw::prelude::*;
use adw::{ActionRow, ApplicationWindow, CenteringPolicy, HeaderBar, ViewStack, ViewSwitcherTitle};
use gtk::{Align, Application, Box, Button, Label, ListBox, MenuButton, Orientation};

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

fn build_header_bar(stack: &ViewStack) -> HeaderBar {
    let header_bar = HeaderBar::builder()
        .centering_policy(CenteringPolicy::Strict)
        .build();

    let btn_add = Button::builder().icon_name("list-add-symbolic").build();
    let btn_menu = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .build();

    header_bar.pack_start(&btn_add);
    header_bar.pack_end(&btn_menu);

    let view_switcher_title = ViewSwitcherTitle::builder()
        .stack(stack)
        .title("Buttercream")
        .build();

    header_bar.set_title_widget(Some(&view_switcher_title));

    header_bar
}

fn build_ui(app: &Application) {
    let view_stack = ViewStack::builder().build();
    let header_bar = build_header_bar(&view_stack);

    let snapshot_list = ListBox::builder().build();

    let remove_btn = Button::builder()
        .icon_name("edit-delete-symbolic")
        .valign(Align::Center)
        .css_classes(vec!["circular".to_string(), "flat".to_string()])
        .build();
    let snapshot_row1 = ActionRow::builder()
        .title("2022-01-17 22:00")
        .subtitle("13 GB")
        .build();
    snapshot_row1.add_suffix(&remove_btn);
    snapshot_list.append(&snapshot_row1);
    let snapshot_page = view_stack.add(&snapshot_list);
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

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Buttercream")
        .content(&content)
        .build();
    window.present();
}
