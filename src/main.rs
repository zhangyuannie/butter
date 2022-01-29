mod application;
mod config;
mod requester;
mod snapshot_object;
mod snapshot_view;
mod ui;
mod window;

use adw::prelude::*;
use requester::daemon;
use std::io;
use std::process::{Command, Stdio};

fn main() {
    let daemon_process = Command::new("pkexec")
        .arg("butterd")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut d = daemon();
        d.set_reader(io::BufReader::new(daemon_process.stdout.unwrap()));
        d.set_writer(daemon_process.stdin.unwrap())
    }

    let app = application::Application::new();
    app.run();
}
