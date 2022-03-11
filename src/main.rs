mod application;
mod config;
mod rename_popover;
mod requester;
mod snapshot_creation_window;
mod snapshot_object;
mod snapshot_view;
mod ui;
mod window;

use adw::prelude::*;
use gtk::gio;
use requester::daemon;
use std::io;
use std::process::{Command, Stdio};

fn main() {
    let daemon_process = Command::new("pkexec")
        .arg(config::DAEMON_EXEC_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut d = daemon();
        d.set_reader(io::BufReader::new(daemon_process.stdout.unwrap()));
        d.set_writer(daemon_process.stdin.unwrap())
    }

    gettext::setlocale(gettext::LocaleCategory::LcAll, "");
    gettext::bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR)
        .expect("Unable to bind the text domain");
    gettext::bind_textdomain_codeset(config::GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to bind text domain codeset");
    gettext::textdomain(config::GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let res = gio::Resource::load(config::GRESOURCE_FILE).expect("Unable to load gresource file");
    gio::resources_register(&res);

    let app = application::Application::new();
    app.run();
}
