use std::convert::AsRef;
use std::fs::{walk_dir, PathExt};
use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, BTreeSet};
use std::slice::SliceConcatExt;
use std::fmt::Display;

use cronparse::{CrontabFile, CrontabFileError, CrontabFileErrorKind, Limited};
use cronparse::crontab::{EnvVarEntry, CrontabEntry, ToCrontabEntry};
use cronparse::crontab::{SystemCrontabEntry, UserCrontabEntry};
use cronparse::schedule::{Schedule, Period, Calendar, DayOfWeek, Month, Day, Hour, Minute};
use cronparse::interval::Interval;
use log::{Logger, LogLevel};

pub fn process_crontab_dir<T: ToCrontabEntry>(dir: &str, logger: &mut Logger) {
    let files = walk_dir(dir).and_then(|fs| fs.map(|r| r.map(|p| p.path()))
                                       .filter(|r| r.as_ref().map(|p| p.is_file()).unwrap_or(true))
                                       .collect::<Result<Vec<PathBuf>, _>>());
    match files {
        Err(err) => log!(logger, LogLevel::Error, "error processing directory {}: {}", dir, err),
        Ok(files) => for file in files {
            process_crontab_file::<T, _>(file, logger as &mut Logger);
        }
    }
}


pub fn process_crontab_file<T: ToCrontabEntry, P: AsRef<Path>>(path: P, logger: &mut Logger) {
    CrontabFile::<T>::new(path.as_ref()).map(|crontab| {
        let mut env = BTreeMap::new();
        for entry in crontab {
            match entry {
                Ok(CrontabEntry::EnvVar(EnvVarEntry(name, value))) => { env.insert(name, value); },
                Ok(data) => generate_systemd_units(path.as_ref(), data, &env, logger),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Io(_), .. }) => log!(logger, LogLevel::Warning, "error accessing file {}: {}", path.as_ref().display(), err),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Parse(_), .. }) => log!(logger, LogLevel::Warning, "skipping file {} due to parsing error: {}", path.as_ref().display(), err),
            }
        }
    }).unwrap_or_else(|err| {
        log!(logger, LogLevel::Error, "error parsing file {}: {}", path.as_ref().display(), err);
    });
}

#[allow(non_snake_case)]
fn generate_systemd_units(path: &Path, entry: CrontabEntry, env: &BTreeMap<String, String>, logger: &mut Logger) {
    use cronparse::crontab::CrontabEntry::*;

    log!(logger, LogLevel::Info, "{} => {:?}, {:?}", path.display(), entry, env);

    let mut PERSISTENT = env.get("PERSISTENT").and_then(|v| match &**v {
        "yes" | "true" | "1" => Some(true),
        "auto" | "" => None,
        _ => Some(false)
    }).unwrap_or_else(|| match entry {
        Anacron(_) | User(UserCrontabEntry { sched: Schedule::Period(_), .. }) | System(SystemCrontabEntry { sched: Schedule::Period(_), .. }) => true,
        _ => false
    });

    let BATCH = env.get("BATCH").map(|v| match &**v {
        "yes" | "true" | "1" => true,
        _ => false
    }).unwrap_or(false);

    let RANDOM_DELAY = env.get("RANDOM_DELAY").and_then(|v| v.parse::<u64>().ok()).unwrap_or(1);
    let mut BOOT_DELAY = env.get("DELAY").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let START_HOURS_RANGE = env.get("START_HOURS_RANGE").and_then(|v| v.splitn(1, '-').next().and_then(|v| v.parse::<u64>().ok())).unwrap_or(0);

    let schedule = entry.period().and_then(|period| match *period {
        Period::Reboot => {
            PERSISTENT = false;
            if BOOT_DELAY == 0 {
                BOOT_DELAY = 1;
            }
            None
        },
        Period::Minutely => {
            PERSISTENT = false;
            Some("@minutely".to_string())
        },
        Period::Hourly => {
            if BOOT_DELAY == 0 {
                Some("@hourly".to_string())
            } else {
                Some(format!("*-*-* *:{}:0", BOOT_DELAY))
            }
        },
        Period::Midnight | Period::Daily => {
            if BOOT_DELAY == 0 {
                Some("@daily".to_string())
            } else {
                Some(format!("*-*-* 0:{}:0", BOOT_DELAY))
            }
        },
        Period::Weekly => {
            if BOOT_DELAY == 0 {
                Some("@weekly".to_string())
            } else {
                Some(format!("Mon *-*-* 0:{}:0", BOOT_DELAY))
            }
        },
        Period::Monthly => {
            if BOOT_DELAY == 0 {
                Some("@monthly".to_string())
            } else {
                Some(format!("*-*-1 0:{}:0", BOOT_DELAY))
            }
        },
        Period::Quaterly => {
            if BOOT_DELAY == 0 {
                Some("@quaterly".to_string())
            } else {
                Some(format!("*-1,4,7,10-1 0:{}:0", BOOT_DELAY))
            }
        },
        Period::Biannually => {
            if BOOT_DELAY == 0 {
                Some("@semi-annually".to_string())
            } else {
                Some(format!("*-1,7-1 0:{}:0", BOOT_DELAY))
            }
        },
        Period::Yearly => {
            if BOOT_DELAY == 0 {
                Some("@yearly".to_string())
            } else {
                Some(format!("*-1-1 0:{}:0", BOOT_DELAY))
            }
        },
        Period::Days(days) => {
            // workaround for anacrontab
            if days > 31 {
                Some(format!("*-1/{}-1 0:{}:0", days / 30, BOOT_DELAY))
            } else {
                Some(format!("*-*-1/{} 0:{}:0", days, BOOT_DELAY))
            }
        },
    }).or_else(|| entry.calendar().and_then(|cal| {
        let Calendar {
            ref dows,
            ref days,
            ref mons,
            ref hrs,
            ref mins
        } = *cal;

        Some(format!("{} *-{}-{} {}:{}:00",
                     linearize(&**dows),
                     linearize(&**mons),
                     linearize(&**days),
                     linearize(&**hrs),
                     linearize(&**mins)))
    }));

    println!("schedule: {:?}", schedule);
}

fn linearize<T: Limited + Display>(input: &[Interval<T>]) -> String {
    if input.len() == 1 && input[0] == Interval::Full(1) {
        "*".to_string()
    } else {
        let mut output = String::new();
        let _ = input.iter().flat_map(|v| v.iter()).collect::<BTreeSet<_>>().iter().scan(&mut output, |acc, item| {
        //let _ = input.iter().scan(&mut output, |acc, item| {
            acc.push_str(&*item.to_string());
            acc.push(',');
            Some(0)
        }).fold(0, |_, _| 0);
        output.pop();
        output
    }
}
