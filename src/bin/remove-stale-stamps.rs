extern crate time;
extern crate glob;

use std::fs::{metadata, remove_file};
use std::path::{Path, PathBuf};
use std::os::unix::fs::MetadataExt;
use std::collections::BTreeSet;

use time::{Duration, get_time};
use glob::glob;

static KNOWN_STAMPS: [&'static str; 6] = ["/var/lib/systemd/timers/stamp-cron-daily.timer",
                                          "/var/lib/systemd/timers/stamp-cron-weekly.timer",
                                          "/var/lib/systemd/timers/stamp-cron-monthly.timer",
                                          "/var/lib/systemd/timers/stamp-cron-quarterly.timer",
                                          "/var/lib/systemd/timers/stamp-cron-semi-annually.timer",
                                          "/var/lib/systemd/timers/stamp-cron-yearly.timer"];

static ACTUAL_STAMPS_GLOB: &'static str = "/var/lib/systemd/timers/stamp-cron-*.timer";
static TIMER_STAMPS_GLOB: &'static str = "/run/systemd/generator/cron-*.timer";

fn cleanup<P: AsRef<Path>, I: IntoIterator<Item = P>>(iter: I) {
    let ten_days_ago = get_time() - Duration::days(10);
    for stamp in iter {
        if let Ok(meta) = metadata(&stamp) {
            if (meta.mtime() as i64) < ten_days_ago.sec {
                let _ = remove_file(&stamp);
            }
        }
    }
}

fn main() {
    let stale_stamps = &(&glob(ACTUAL_STAMPS_GLOB)
        .unwrap()
        .flat_map(Result::into_iter)
        .collect::<BTreeSet<_>>() -
                         &glob(TIMER_STAMPS_GLOB)
        .unwrap()
        .flat_map(Result::into_iter)
        .map(|s| {
            PathBuf::from(s.to_string_lossy()
                           .replace("/run/systemd/generator/cron-", "/var/lib/systemd/timers/stamp-cron-"))
        })
        .collect::<BTreeSet<_>>()) -
                       &KNOWN_STAMPS.iter()
                                    .map(PathBuf::from)
                                    .collect::<BTreeSet<_>>();

    cleanup(&stale_stamps);
}
