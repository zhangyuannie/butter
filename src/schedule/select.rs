use chrono::{Datelike, NaiveDateTime, Timelike};

use super::conf::Schedule;

pub trait DateTimeSortable: Ord {
    fn created(&self) -> &NaiveDateTime;
}

pub fn select_snapshots_to_remove<T: DateTimeSortable>(
    mut snapshots: Vec<T>,
    schedule: &Schedule,
) -> Vec<T> {
    snapshots.sort();
    snapshots.reverse();
    struct Bucket {
        keep: u32,
        last: i32,
        algo: fn(&NaiveDateTime) -> i32,
    }

    let mut ret = Vec::new();
    let mut buckets = [
        Bucket {
            keep: schedule.keep_hourly,
            last: 0,
            algo: |dt| dt.year() * 100000 + dt.ordinal() as i32 * 100 + dt.hour() as i32,
        },
        Bucket {
            keep: schedule.keep_daily,
            last: 0,
            algo: |dt| dt.year() * 1000 + dt.ordinal() as i32,
        },
        Bucket {
            keep: schedule.keep_weekly,
            last: 0,
            algo: |dt| {
                let week = dt.iso_week();
                week.year() * 100 + week.week() as i32
            },
        },
        Bucket {
            keep: schedule.keep_monthly,
            last: 0,
            algo: |dt| dt.year() * 100 + dt.month() as i32,
        },
        Bucket {
            keep: schedule.keep_yearly,
            last: 0,
            algo: |dt| dt.year(),
        },
    ];

    for snapshot in snapshots {
        let mut should_remove = true;
        for bucket in &mut buckets {
            if bucket.keep > 0 {
                let val = (bucket.algo)(&snapshot.created());
                if val != bucket.last {
                    bucket.keep -= 1;
                    bucket.last = val;
                    should_remove = false;
                }
            }
        }

        if should_remove {
            ret.push(snapshot);
        }
    }

    ret
}
