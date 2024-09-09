use std::{
    io::{self, Error, ErrorKind},
    path::PathBuf,
};

pub struct MntEntries<R: io::BufRead> {
    reader: R,
}

impl<R: io::BufRead> MntEntries<R> {
    pub fn new(reader: R) -> MntEntries<R> {
        MntEntries { reader }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MntEntry {
    pub spec: String,
    pub target: Option<PathBuf>,
    pub fs_type: String,
    pub options: String,
    pub dump_freq: i32,
    pub pass: i32,
}

impl<R: io::BufRead> Iterator for MntEntries<R> {
    type Item = io::Result<MntEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(0) => return None,
                Ok(_) => {
                    if buf.is_empty() || buf.starts_with("#") {
                        // skip comment and empty line
                        continue;
                    } else {
                        let mut words = buf.split_ascii_whitespace();

                        return Some(Ok(MntEntry {
                            spec: match words.next() {
                                Some(s) => s.to_string(),
                                None => return Some(Err(Error::from(ErrorKind::InvalidData))),
                            },
                            target: match words.next() {
                                Some(s) => {
                                    if s == "none" {
                                        None
                                    } else {
                                        // TODO: escapes
                                        Some(PathBuf::from(s))
                                    }
                                }
                                None => return Some(Err(Error::from(ErrorKind::InvalidData))),
                            },
                            fs_type: match words.next() {
                                Some(s) => s.to_string(),
                                None => return Some(Err(Error::from(ErrorKind::InvalidData))),
                            },
                            options: match words.next() {
                                Some(s) => s.to_string(),
                                None => return Some(Err(Error::from(ErrorKind::InvalidData))),
                            },
                            dump_freq: match words.next() {
                                Some(s) => s.parse().unwrap_or(0),
                                None => 0,
                            },
                            pass: match words.next() {
                                Some(s) => s.parse().unwrap_or(0),
                                None => 0,
                            },
                        }));
                    }
                }
                Err(e) => return Some(Err(e)),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btrfs_filesystem_show_impl() {
        let input = b"proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0\n\
            sysfs /sys sysfs rw,seclabel,nosuid,nodev,noexec,relatime 0 0\n\
            /dev/nvme0n1p2 /home btrfs rw,seclabel,relatime,compress=zstd:1,ssd,space_cache,subvolid=256,subvol=/home 0 0\n"
            as &[u8];

        let mut ent = MntEntries::new(input);
        assert_eq!(
            ent.next().unwrap().unwrap(),
            MntEntry {
                spec: "proc".to_string(),
                target: Some("/proc".into()),
                fs_type: "proc".to_string(),
                options: "rw,nosuid,nodev,noexec,relatime".to_string(),
                dump_freq: 0,
                pass: 0
            }
        );
        assert_eq!(
            ent.next().unwrap().unwrap(),
            MntEntry {
                spec: "sysfs".to_string(),
                target: Some("/sys".into()),
                fs_type: "sysfs".to_string(),
                options: "rw,seclabel,nosuid,nodev,noexec,relatime".to_string(),
                dump_freq: 0,
                pass: 0
            }
        );
        assert_eq!(
            ent.next().unwrap().unwrap(),
            MntEntry {
                spec: "/dev/nvme0n1p2".to_string(),
                target: Some("/home".into()),
                fs_type: "btrfs".to_string(),
                options:
                    "rw,seclabel,relatime,compress=zstd:1,ssd,space_cache,subvolid=256,subvol=/home"
                        .to_string(),
                dump_freq: 0,
                pass: 0
            }
        );
        assert!(ent.next().is_none());
    }
}
