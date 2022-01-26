use dbus::blocking::Connection;
use dbus_crossroads::{Crossroads, IfaceBuilder};
use std::os::unix::process::ExitStatusExt;
use std::process::Command;

fn run(cmd: &str, args: Vec<String>) -> (i32, String) {
    Command::new(cmd)
        .args(args)
        .output()
        .map_or((127, "".into()), |r| {
            (
                r.status
                    .code()
                    .unwrap_or_else(|| r.status.signal().unwrap_or(0) + 128),
                String::from_utf8(r.stdout).unwrap(),
            )
        })
}

fn main() {
    let c = Connection::new_system().unwrap();
    c.request_name("org.zhangyuannie.butter", false, true, false)
        .unwrap();

    let mut cr = Crossroads::new();

    let iface_token = cr.register(
        "org.zhangyuannie.butter",
        |builder: &mut IfaceBuilder<()>| {
            builder.method(
                "RunBtrfs",
                ("args",),
                ("status", "stdout"),
                move |_, _, (args,): (Vec<String>,)| Ok(run("btrfs", args)),
            );
        },
    );
    cr.insert("/", &[iface_token], ());

    cr.serve(&c).unwrap();
    unreachable!()
}
