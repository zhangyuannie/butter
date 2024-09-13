use butterd::{create_snapshot, RuleConfig, RuleSubvolumeConfig, SnapshotMetadata};

use std::{cmp, fs, io, os::unix::prelude::OsStrExt, path::PathBuf};

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use log;

mod name {
    use rand::{prelude::ThreadRng, Rng, RngCore};

    const ADJECTIVES: &[&str] = &[
        "amazing", "aquatic", "artistic", "awesome", "big", "bold", "brave", "busy", "calm",
        "charming", "cool", "dynamic", "elegant", "friendly", "great", "honest", "innocent",
        "jovial", "kind", "lucky", "magical", "nice", "optimal", "precious", "quiet", "relaxed",
        "sweet", "talented", "ultimate", "vigilant", "wild", "xenial", "yellow", "zen",
    ];

    const NAMES: &[&str] = &[
        "alpaca", "ant", "ape", "beaver", "bison", "bug", "cat", "coyote", "crow", "deer", "dog",
        "duck", "eagle", "fish", "flamingo", "giraffe", "hamster", "horse", "ibex", "jaguar",
        "kangaroo", "koala", "llama", "leopard", "lobster", "lynx", "moose", "nautilus", "octopus",
        "owl", "ox", "panda", "penguin", "pigeon", "pony", "quagga", "rabbit", "rhino", "salmon",
        "sheep", "snake", "turtle", "unicorn", "vole", "wolf",
    ];

    pub struct RandomName {
        buf: String,
        rng: ThreadRng,
        has_postfix: bool,
    }

    impl RandomName {
        pub fn new() -> Self {
            let mut rng = rand::thread_rng();
            let adj = ADJECTIVES[rng.gen_range(0..ADJECTIVES.len())];
            let name = NAMES[rng.gen_range(0..NAMES.len())];

            let mut buf = String::with_capacity(adj.len() + name.len() + 1);

            buf.push_str(adj);
            buf.push('_');
            buf.push_str(name);

            RandomName {
                buf,
                rng,
                has_postfix: false,
            }
        }

        pub fn inc_len(&mut self) {
            const CHARSET: &[u8; 32] = b"abcdefghijkmnpqrstuvwxyz23456789";
            if !self.has_postfix {
                self.buf.push('_');
                self.has_postfix = true;
            }
            let idx = self.rng.next_u32() >> (32 - 5);
            self.buf.push(CHARSET[idx as usize] as char)
        }

        pub fn as_str(&self) -> &str {
            &self.buf
        }
    }
}

fn should_prune(c: &RuleConfig) -> bool {
    c.keep_hourly != 0
        || c.keep_daily != 0
        || c.keep_weekly != 0
        || c.keep_monthly != 0
        || c.keep_yearly != 0
}

pub fn snapshot(c: &RuleConfig) {
    for subvol in &c.subvolumes {
        log::info!(
            "creating a snapshot from '{}' in '{}'",
            subvol.path.display(),
            subvol.target_dir.display()
        );
        let ret = snapshot_subvol(subvol);
        if let Err(e) = ret {
            log::error!(
                "failed to create a snapshot from '{}': {}",
                subvol.path.display(),
                e
            );
        }
    }
}
pub fn prune(c: &RuleConfig) {
    if !should_prune(c) {
        return;
    }

    for subvol in &c.subvolumes {
        let res = prune_subvol(subvol, c);
        if let Err(err) = res {
            eprintln!("failed to prune '{}': {}", subvol.target_dir.display(), err);
        }
    }
}

fn snapshot_subvol(c: &RuleSubvolumeConfig) -> anyhow::Result<()> {
    let mut name = name::RandomName::new();
    for _ in 0..16 {
        let target_path = c.target_dir.join(name.as_str());
        match create_snapshot(&c.path, &target_path, true) {
            Ok(_) => return Ok(()),
            Err(e) => {
                if e.kind() == io::ErrorKind::AlreadyExists {
                    name.inc_len();
                    continue;
                } else {
                    return Err(e.into());
                }
            }
        }
    }
    Err(anyhow::anyhow!("name exhausted"))
}

fn prune_subvol(subvol_cfg: &RuleSubvolumeConfig, rule_cfg: &RuleConfig) -> anyhow::Result<()> {
    #[derive(Debug, PartialEq, Eq)]
    struct Snapshot {
        created: NaiveDateTime,
        path: PathBuf,
    }
    struct Bucket {
        keep: u32,
        last: i32,
        algo: fn(&NaiveDateTime) -> i32,
    }

    let source_subvol_path = libbtrfsutil::subvolume_path(&subvol_cfg.path)?;

    let mut snapshots: Vec<Snapshot> = fs::read_dir(&subvol_cfg.target_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_name().as_bytes()[0] == b'.' {
                return None;
            }
            let path = entry.path();
            let info = libbtrfsutil::subvolume_info(&path).ok()?;
            if let Some(metadata) = SnapshotMetadata::read(&path) {
                if metadata.created_from == source_subvol_path {
                    return Some(Snapshot {
                        path,
                        created: DateTime::<Utc>::from(info.created()).naive_local(),
                    });
                }
            }
            return None;
        })
        .collect();
    snapshots.sort_by_key(|e| cmp::Reverse(e.created));

    let mut buckets = [
        Bucket {
            keep: rule_cfg.keep_hourly,
            last: 0,
            algo: |dt| dt.year() * 100000 + dt.ordinal() as i32 * 100 + dt.hour() as i32,
        },
        Bucket {
            keep: rule_cfg.keep_daily,
            last: 0,
            algo: |dt| dt.year() * 1000 + dt.ordinal() as i32,
        },
        Bucket {
            keep: rule_cfg.keep_weekly,
            last: 0,
            algo: |dt| {
                let week = dt.iso_week();
                week.year() * 100 + week.week() as i32
            },
        },
        Bucket {
            keep: rule_cfg.keep_monthly,
            last: 0,
            algo: |dt| dt.year() * 100 + dt.month() as i32,
        },
        Bucket {
            keep: rule_cfg.keep_yearly,
            last: 0,
            algo: |dt| dt.year(),
        },
    ];

    for snapshot in snapshots {
        let mut should_remove = true;
        for bucket in &mut buckets {
            if bucket.keep > 0 {
                let val = (bucket.algo)(&snapshot.created);
                if val != bucket.last {
                    bucket.keep -= 1;
                    bucket.last = val;
                    should_remove = false;
                }
            }
        }

        if should_remove {
            println!("deleting '{}'", snapshot.path.display());
            let res = libbtrfsutil::DeleteSubvolumeOptions::new()
                .recursive(true)
                .delete(&snapshot.path);
            if let Err(err) = res {
                eprintln!("failed to delete '{}': {}", snapshot.path.display(), err);
            }
        }
    }

    Ok(())
}
