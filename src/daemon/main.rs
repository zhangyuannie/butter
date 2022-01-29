use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::{io, process};

fn run(cmd: &str, args: Vec<String>) -> (i32, String) {
    process::Command::new(cmd)
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
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        let mut req = String::new();
        let len = stdin.read_line(&mut req).unwrap();
        if len == 0 {
            break;
        }
        let args: Vec<String> = serde_json::from_str(&req).unwrap();
        let reply = run("btrfs", args);
        writeln!(stdout, "{}", serde_json::to_string(&reply).unwrap()).unwrap();
    }
}
