#![feature(path_ext)]
#![feature(fs_walk)]
#![feature(convert)]

#![allow(dead_code)] // for debug

#[macro_use(log)]
extern crate cronparse;

use std::convert::AsRef;
use std::fs::{walk_dir, PathExt};
use std::path::{Path, PathBuf};
use std::thread::spawn;
use std::collections::BTreeMap;
use std::env;

use cronparse::{CrontabFile, CrontabFileError, CrontabFileErrorKind};
use cronparse::crontab::{UserCrontabEntry, SystemCrontabEntry, AnacrontabEntry, EnvVarEntry, CrontabEntry, ToCrontabEntry};
use cronparse::log::{KernelLogger, ConsoleLogger, AnyLogger, Logger, LogLevel};

fn generate_systemd_units(path: &Path, entry: CrontabEntry, env: &BTreeMap<String, String>) {
    println!("{} => {:?}, {:?}", path.display(), entry, env);
}

static USERS_CRONTAB_DIR: &'static str = "/var/spool/cron";  // UserCrontabEntry
static SYSTEM_CRONTAB_DIR: &'static str = "/etc/cron.d";  // SystemCrontabEntry
static ANACRONTAB_FILE: &'static str = "/etc/anacrontab";  // AnacrontabEntry

fn process_crontab_file<T: ToCrontabEntry, P: AsRef<Path>>(path: P, logger: &mut Logger) {
    CrontabFile::<T>::new(path.as_ref()).map(|crontab| {
        let mut env = BTreeMap::new();
        for entry in crontab {
            match entry {
                Ok(CrontabEntry::EnvVar(EnvVarEntry(name, value))) => { env.insert(name, value); },
                Ok(data) => generate_systemd_units(path.as_ref(), data, &env),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Io(_), .. }) => log!(logger, LogLevel::Warning, "error accessing file {}: {}", path.as_ref().display(), err),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Parse(_), .. }) => log!(logger, LogLevel::Warning, "skipping file {} due to parsing error: {}", path.as_ref().display(), err),
            }
        }
    }).unwrap_or_else(|err| {
        log!(logger, LogLevel::Error, "error parsing file {}: {}", path.as_ref().display(), err);
    });
}

#[inline]
fn is_run_by_systemd() -> bool {
    env::args().len() >= 3
}

fn get_logger() -> AnyLogger {
    if is_run_by_systemd() {
        AnyLogger::Kernel(KernelLogger::new())
    } else {
        AnyLogger::Console(ConsoleLogger::new())
    }
}

fn process_crontab_dir<T: ToCrontabEntry>(dir: &str, logger: &mut Logger) {
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

fn main() {
    let user_thread = spawn(|| process_crontab_dir::<UserCrontabEntry>(USERS_CRONTAB_DIR, &mut get_logger()));
    let system_thread = spawn(|| process_crontab_dir::<SystemCrontabEntry>(SYSTEM_CRONTAB_DIR, &mut get_logger()));
    let anacron_thread = spawn(|| process_crontab_file::<AnacrontabEntry, _>(ANACRONTAB_FILE, &mut get_logger()));

    let _ = user_thread.join();
    let _ = system_thread.join();
    let _ = anacron_thread.join();
}
