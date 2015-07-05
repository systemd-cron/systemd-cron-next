use std::convert::AsRef;
use std::fs::{walk_dir, PathExt, File, create_dir_all};
use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{symlink, MetadataExt};
use std::io::Write;

use cronparse::{CrontabFile, CrontabFileError, CrontabFileErrorKind, Limited};
use cronparse::crontab::{EnvVarEntry, CrontabEntry, ToCrontabEntry};
use cronparse::crontab::{SystemCrontabEntry, UserCrontabEntry};
use cronparse::schedule::{Schedule, Period, Calendar};
use cronparse::interval::Interval;

fn tohex(input: &[u8]) -> String {
    #[inline]
    fn hex(d: u8) -> char {
        match d {
            0...9 => (d + 0x30) as char,
            10...15 => (d + 0x57) as char,
            _ => unreachable!("unexpected value: {}", d)
        }
    }

    let mut buf = String::with_capacity(32);
    for b in input.into_iter() {
        buf.push(hex(b >> 4));
        buf.push(hex(b & 0xf));
    }
    buf
}


pub fn process_crontab_dir<T: ToCrontabEntry, D: AsRef<Path>>(srcdir: &str, dstdir: D) {
    let files = walk_dir(srcdir).and_then(|fs| fs.map(|r| r.map(|p| p.path()))
                                       .filter(|r| r.as_ref().map(|p| p.is_file()).unwrap_or(true))
                                       .collect::<Result<Vec<PathBuf>, _>>());
    match files {
        Err(err) => error!("error processing directory {}: {}", srcdir, err),
        Ok(files) => for file in files {
            process_crontab_file::<T, _, _>(file, dstdir.as_ref());
        }
    }
}


pub fn process_crontab_file<T: ToCrontabEntry, P: AsRef<Path>, D: AsRef<Path>>(path: P, dstdir: D) {
    CrontabFile::<T>::new(path.as_ref()).map(|crontab| {
        let mut env = BTreeMap::new();
        for entry in crontab {
            match entry {
                Ok(CrontabEntry::EnvVar(EnvVarEntry(name, value))) => { env.insert(name, value); },
                Ok(data) => generate_systemd_units(data, &env, path.as_ref(), dstdir.as_ref()),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Io(_), .. }) => warn!("error accessing file {}: {}", path.as_ref().display(), err),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Parse(_), .. }) => warn!("skipping file {} due to parsing error: {}", path.as_ref().display(), err),
            }
        }
    }).unwrap_or_else(|err| {
        error!("error parsing file {}: {}", path.as_ref().display(), err);
    });
}

macro_rules! try_ {
    ($exp:expr) => {
        match $exp {
            Ok(value) => value,
            Err(err) => { error!("{}", err); return; }
        }
    }
}

#[allow(non_snake_case)]
fn generate_systemd_units(entry: CrontabEntry, env: &BTreeMap<String, String>, path: &Path, dstdir: &Path) {
    use cronparse::crontab::CrontabEntry::*;

    info!("generating units for {}: {:?}, {:?}", path.display(), entry, env);

    let owner = try_!(path.metadata()).uid();

    let mut persistent = env.get("PERSISTENT").and_then(|v| match &**v {
        "yes" | "true" | "1" => Some(true),
        "auto" | "" => None,
        _ => Some(false)
    }).unwrap_or_else(|| match entry {
        Anacron(_) | User(UserCrontabEntry { sched: Schedule::Period(_), .. }) | System(SystemCrontabEntry { sched: Schedule::Period(_), .. }) => true,
        _ => false
    });

    let batch = env.get("BATCH").map(|v| match &**v {
        "yes" | "true" | "1" => true,
        _ => false
    }).unwrap_or(false);

    let random_delay = env.get("RANDOM_DELAY").and_then(|v| v.parse::<u64>().ok()).unwrap_or(1);
    let mut delay = env.get("DELAY").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let hour = env.get("START_HOURS_RANGE").and_then(|v| v.splitn(1, '-').next().and_then(|v| v.parse::<u64>().ok())).unwrap_or(0);

    let schedule = entry.period().and_then(|period| match *period {
        Period::Reboot => {
            persistent = false;
            if delay == 0 {
                delay = 1;
            }
            None
        },
        Period::Minutely => {
            persistent = false;
            Some("@minutely".to_owned())
        },
        Period::Hourly => {
            if delay == 0 {
                Some("@hourly".to_owned())
            } else {
                Some(format!("*-*-* *:{}:0", delay))
            }
        },
        Period::Midnight => {
            if delay == 0 {
                Some("@daily".to_owned())
            } else {
                Some(format!("*-*-* 0:{}:0", delay))
            }
        },
        Period::Daily => {
            if delay == 0 && hour == 0 {
                Some("@daily".to_owned())
            } else {
                Some(format!("*-*-* {}:{}:0", hour, delay))
            }
        },
        Period::Weekly => {
            if delay == 0 && hour == 0 {
                Some("@weekly".to_owned())
            } else {
                Some(format!("Mon *-*-* {}:{}:0", hour, delay))
            }
        },
        Period::Monthly => {
            if delay == 0 && hour == 0 {
                Some("@monthly".to_owned())
            } else {
                Some(format!("*-*-1 {}:{}:0", hour, delay))
            }
        },
        Period::Quaterly => {
            if delay == 0 && hour == 0 {
                Some("@quaterly".to_owned())
            } else {
                Some(format!("*-1,4,7,10-1 {}:{}:0", hour, delay))
            }
        },
        Period::Biannually => {
            if delay == 0 && hour == 0 {
                Some("@semi-annually".to_owned())
            } else {
                Some(format!("*-1,7-1 {}:{}:0", hour, delay))
            }
        },
        Period::Yearly => {
            if delay == 0 && hour == 0 {
                Some("@yearly".to_owned())
            } else {
                Some(format!("*-1-1 {}:{}:0", hour, delay))
            }
        },
        Period::Days(days) => {
            // workaround for anacrontab
            if days > 31 {
                Some(format!("*-1/{}-1 {}:{}:0", days / 30, hour, delay))
            } else {
                Some(format!("*-*-1/{} {}:{}:0", days, hour, delay))
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

    if let Some(command) = entry.command() {
        let mut md5ctx = ::md5::Context::new();
        md5ctx.consume(path.as_os_str().as_bytes());
        if let Some(ref schedule) = schedule {
            md5ctx.consume(schedule.as_bytes());
        }
        md5ctx.consume(command.as_bytes());
        let md5hex = tohex(&md5ctx.compute());

        let service_unit_name = format!("cronjob-{}.service", md5hex);
        let timer_unit_name = format!("cronjob-{}.timer", md5hex);

        let service_unit_path = dstdir.join(service_unit_name);
        let timer_unit_path = dstdir.join(&timer_unit_name);
        let cron_target_wants_path = dstdir.join("cron.target.wants");
        try_!(create_dir_all(&cron_target_wants_path));

        debug!("generating timer {:?} from {:?}", timer_unit_path, path);
        {
            let mut timer_unit_file = try_!(File::create(&timer_unit_path));

            try_!(writeln!(timer_unit_file, r###"[Unit]
Description=[Timer] "{entry}"
Documentation=man:systemd-crontab-generator(8)
PartOf=cron.target
RefuseManualStart=true
RefuseManualStop=true
SourcePath={source_crontab_path}

[Timer]
Unit={service_unit_path}
Persistent={persistent}"###,
                entry = entry,
                source_crontab_path = path.display(),
                service_unit_path = service_unit_path.display(),
                persistent = persistent,
                ));

            if let Some(schedule) = schedule {
                try_!(writeln!(timer_unit_file, "OnCalendar={}", schedule));
            } else {
                try_!(writeln!(timer_unit_file, "OnBootSec={}m", delay));
            }

            if random_delay != 1 {
                try_!(writeln!(timer_unit_file, "AccuracySec={}m", random_delay));
            }
        }
        try_!(symlink(timer_unit_path, cron_target_wants_path.join(timer_unit_name)));

        debug!("generating service {:?} from {:?}", service_unit_path, path);
        {
            let mut service_unit_file = try_!(File::create(service_unit_path));

            try_!(writeln!(service_unit_file, r###"[Unit]
Description=[Cron] "{entry}"
Documentation=man:systemd-crontab-generator(8)
RefuseManualStart=true
RefuseManualStop=true
SourcePath={source_crontab_path}

[Service]
Type=oneshot
IgnoreSIGPIPE=false
ExecStart={command}"###,
                entry = entry,
                source_crontab_path = path.display(),
                command = command,
                ));

            if let Some(user) = entry.user() {
                try_!(writeln!(service_unit_file, "User={}", user));
            } else if owner != 0 {
                try_!(writeln!(service_unit_file, "User={}", owner));
            }

            if let Some(group) = entry.group() {
                try_!(writeln!(service_unit_file, "Group={}", group));
            }
            if batch {
                try_!(writeln!(service_unit_file, "CPUSchedulingPolicy=idle"));
                try_!(writeln!(service_unit_file, "IOSchedulingClass=idle"));
            }

            if !env.is_empty() {
                try_!(write!(service_unit_file, "Environment="));
                for (name, value) in env.iter() {
                    try_!(write!(service_unit_file, r#""{}={}""#, name, value));
                }
                try_!(write!(service_unit_file, "\n"));
            }
        }
    }
}

fn linearize<T: Limited + Display>(input: &[Interval<T>]) -> String {
    if input.len() == 1 && input[0] == Interval::Full(1) {
        "*".to_owned()
    } else {
        let mut output = String::new();
        for part in input.iter().flat_map(|v| v.iter()).collect::<BTreeSet<_>>().iter() {
            output.push_str(&*part.to_string());
            output.push(',');
        }
        output.pop();
        output
    }
}
