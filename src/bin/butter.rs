use butter::{
    config,
    rule::ReadRuleDir,
    ui::{store::Store, Application},
};
use clap::{Parser, Subcommand};
use gtk::{gio, prelude::*};

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
    let store = Store::new().expect("Failed to connect to system dbus");

    gettext::setlocale(gettext::LocaleCategory::LcAll, "");
    gettext::bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR)
        .expect("Unable to bind the text domain");
    gettext::bind_textdomain_codeset(config::GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to bind text domain codeset");
    gettext::textdomain(config::GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let res = gio::Resource::load(config::GRESOURCE_FILE).expect("Unable to load gresource file");
    gio::resources_register(&res);

    let app = Application::new(&store);
    app.run();
}

pub fn cmd_snapshot() {
    for schedule in ReadRuleDir::new().expect("Failed to read config directory") {
        if let Ok(schedule) = schedule {
            schedule.snapshot();
        }
    }
}

pub fn cmd_prune() {
    for schedule in ReadRuleDir::new().expect("Failed to read config directory") {
        if let Ok(schedule) = schedule {
            schedule.prune();
        }
    }
}
