mod application;
mod btrfs;
mod config;
mod ui;
mod window;
mod snapshot_view;
mod snapshot_object;

use adw::prelude::*;

fn main() {
    let app = application::Application::new();
    app.run();
}
