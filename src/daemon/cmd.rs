use std::{path::PathBuf, process::Command};

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;

use crate::daemon::interface::BtrfsFilesystem;

pub fn btrfs_filesystem_show() -> Result<Vec<BtrfsFilesystem>> {
    let output = Command::new("btrfs")
        .args(["filesystem", "show"])
        .output()?
        .stdout;
    let output = std::str::from_utf8(&output).unwrap();

    btrfs_filesystem_show_impl(output)
}

fn btrfs_filesystem_show_impl(s: &str) -> Result<Vec<BtrfsFilesystem>> {
    s.split_terminator("\n\n")
        .map(|section| parse_single_fs(section))
        .collect()
}

fn parse_single_fs(s: &str) -> Result<BtrfsFilesystem> {
    debug_assert!(s.len() > 5);
    lazy_static! {
        static ref RE_LABEL: Regex = Regex::new(r"^Label:\s+'(.*)'").unwrap();
        static ref RE_UUID: Regex = Regex::new(r"\suuid:\s+(\S+)\s").unwrap();
        static ref RE_TOTAL_DEVICES: Regex = Regex::new(r"\s*Total devices\s+(\S+)\s+").unwrap();
        static ref RE_PATH: Regex = Regex::new(r"\s+path\s+(.*)$").unwrap();
    }

    let label: Option<String> = if s.starts_with("Label: none") {
        None
    } else {
        let label_match = RE_LABEL
            .captures(s)
            .context("failed to capture fs label")?
            .get(1)
            .context("failed to capture fs label")?;

        Some(label_match.as_str().to_string())
    };

    let uuid = RE_UUID
        .captures(s)
        .context("failed to capture fs uuid")?
        .get(1)
        .context("failed to capture fs uuid")?;
    let uuid = Uuid::parse_str(uuid.as_str()).context("failed to parse uuid")?;

    let mut lines = s.lines();
    let total_device_line = lines
        .find(|line| RE_TOTAL_DEVICES.is_match(line))
        .context("failed to match total devices")?;
    let device_count = RE_TOTAL_DEVICES
        .captures(total_device_line)
        .context("failed to capture total devices")?
        .get(1)
        .context("failed to capture total devices")?;

    let device_count = device_count.as_str().parse()?;
    let mut devices = Vec::with_capacity(device_count);

    let mut matched = 0;
    while matched < device_count {
        if let Some(device) = RE_PATH
            .captures(lines.next().context("not enough devices")?)
            .and_then(|cap| cap.get(1))
        {
            matched += 1;
            devices.push(PathBuf::from(device.as_str()));
        }
    }

    Ok(BtrfsFilesystem {
        label,
        uuid,
        devices,
    })
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_btrfs_filesystem_show_impl() {
        let fedora = "Label: 'fedora'  uuid: 0f3b1507-7dfe-4ef3-bade-71bcfc0fb08e\n\
            \tTotal devices 1 FS bytes used 291.25GiB\n\
            \tdevid    1 size 664.06GiB used 379.02GiB path /dev/nvme0n1p2\n\
            \n";
        assert_eq!(
            btrfs_filesystem_show_impl(fedora).unwrap(),
            vec![BtrfsFilesystem {
                label: Some("fedora".to_string()),
                uuid: Uuid::parse_str("0f3b1507-7dfe-4ef3-bade-71bcfc0fb08e").unwrap(),
                devices: vec![PathBuf::from("/dev/nvme0n1p2")]
            }]
        );

        let multi_dev = "Label: 'RootFS'  uuid: c87975a0-a575-425e-9890-d3f7fcfbbd96\n\
            Total devices 2 FS bytes used 284.98GB\n\
            \tdevid    2 size 311.82GB used 286.51GB used 286.51GB path /dev/sdb1\n\
            \tdevid    1 size 897.76GB used 286.51GB path /dev/sda1\n\
            \n";
        assert_eq!(
            btrfs_filesystem_show_impl(multi_dev).unwrap(),
            vec![BtrfsFilesystem {
                label: Some("RootFS".to_string()),
                uuid: Uuid::parse_str("c87975a0-a575-425e-9890-d3f7fcfbbd96").unwrap(),
                devices: vec![PathBuf::from("/dev/sdb1"), PathBuf::from("/dev/sda1")]
            }]
        );

        let multi_fs = "Label: 'root'  uuid: 84b13cb2-bec5-4deb-8900-7afba3826b32\n\
            \tTotal devices 1 FS bytes used 10.85GiB\n\
            \tdevid    1 size 12.00GiB used 12.00GiB path /dev/vda2\n\
            \n\
            Label: 'home'  uuid: 4f16166a-e77a-4d31-a220-187c510de79e\n\
            \tTotal devices 1 FS bytes used 28.68MiB\n\
            \tdevid    1 size 7.00GiB used 740.00MiB path /dev/vda3\n\
            \n";
        assert_eq!(
            btrfs_filesystem_show_impl(multi_fs).unwrap(),
            vec![
                BtrfsFilesystem {
                    label: Some("root".to_string()),
                    uuid: Uuid::parse_str("84b13cb2-bec5-4deb-8900-7afba3826b32").unwrap(),
                    devices: vec![PathBuf::from("/dev/vda2")],
                },
                BtrfsFilesystem {
                    label: Some("home".to_string()),
                    uuid: Uuid::parse_str("4f16166a-e77a-4d31-a220-187c510de79e").unwrap(),
                    devices: vec![PathBuf::from("/dev/vda3")]
                }
            ]
        );
    }
}
