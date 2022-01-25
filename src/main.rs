mod application;
mod btrfs;
mod config;
mod ui;

use adw::prelude::*;

fn main() {
    let app = application::Application::new();
    app.run();
}
