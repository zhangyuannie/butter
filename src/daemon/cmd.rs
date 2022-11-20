use std::process::Command;

use anyhow::Result;

pub fn is_systemd_unit_active(unit: &str) -> Result<bool> {
    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", unit])
        .status()?;
    Ok(status.success())
}

pub fn enable_systemd_unit(unit: &str, now: bool) -> Result<()> {
    let mut cmd = Command::new("systemctl");
    cmd.args(["enable", unit]);
    if now {
        cmd.arg("--now");
    }

    let output = cmd.output()?;
    if output.status.success() {
        Ok(())
    } else {
        let output = String::from_utf8(output.stderr).unwrap();
        Err(anyhow::Error::msg(output))
    }
}

pub fn disable_systemd_unit(unit: &str, now: bool) -> Result<()> {
    let mut cmd = Command::new("systemctl");
    cmd.args(["disable", unit]);
    if now {
        cmd.arg("--now");
    }

    let output = cmd.output()?;
    if output.status.success() {
        Ok(())
    } else {
        let output = String::from_utf8(output.stderr).unwrap();
        Err(anyhow::Error::msg(output))
    }
}
