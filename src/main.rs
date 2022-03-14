mod application;
mod config;
mod file_chooser_entry;
mod rename_popover;
mod snapshot_column_cell;
mod snapshot_creation_window;
mod snapshot_view;
mod subvolume;
mod subvolume_manager;
mod ui;
mod window;

use adw::prelude::*;
use gtk::gio;
use std::process::{Command, Stdio};
use subvolume_manager::SubvolumeManager;

fn main() {
    let daemon_process = Command::new("pkexec")
        .arg(config::DAEMON_EXEC_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let subvol_manager = SubvolumeManager::new(
        daemon_process.stdin.unwrap(),
        daemon_process.stdout.unwrap(),
    );

    gettext::setlocale(gettext::LocaleCategory::LcAll, "");
    gettext::bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR)
        .expect("Unable to bind the text domain");
    gettext::bind_textdomain_codeset(config::GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to bind text domain codeset");
    gettext::textdomain(config::GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let res = gio::Resource::load(config::GRESOURCE_FILE).expect("Unable to load gresource file");
    gio::resources_register(&res);

    let app = application::Application::new(subvol_manager);
    app.run();
}
