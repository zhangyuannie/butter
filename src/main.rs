mod application;
mod btrfs;
mod config;
mod ui;
mod window;
mod snapshot_view;

use adw::prelude::*;

fn main() {
    let app = application::Application::new();
    app.run();
}
