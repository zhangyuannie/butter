mod application;
mod client;
mod config;
mod daemon;
mod json_file;
mod schedule;
mod schedule_repo;
mod subvolume;
mod ui;
mod widgets;

use crate::schedule::{cmd_prune, cmd_snapshot};
use adw::prelude::*;
use gtk::gio;
use std::process::{Command, Stdio};
use subvolume::SubvolumeManager;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    Schedule {
        #[clap(subcommand)]
        cmd: ScheduleCmd,
    },
}

#[derive(Subcommand)]
enum ScheduleCmd {
    Snapshot,
    Prune,
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Some(Cmd::Schedule { cmd }) => match cmd {
            ScheduleCmd::Snapshot => cmd_snapshot(),
            ScheduleCmd::Prune => cmd_prune(),
        },
        None => gui(),
    }
}

fn gui() {
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
