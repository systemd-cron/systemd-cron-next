#![feature(path_ext)]
#![feature(fs_walk)]
#![feature(convert)]

#![allow(dead_code)] // for debug

extern crate cronparse;

use std::convert::AsRef;
use std::fs::{walk_dir, PathExt};
use std::path::Path;
use std::io;
use std::thread::spawn;

use cronparse::{CrontabFile, CrontabFileError, CrontabFileErrorKind};
use cronparse::crontab::{UserCrontabEntry, SystemCrontabEntry, AnacrontabEntry, EnvVarEntry, CrontabEntry, ToCrontabEntry};

fn generate_systemd_units(path: &Path, entry: CrontabEntry) {
    println!("{} => {:?}", path.display(), entry);
}

static USERS_CRONTAB_DIR: &'static str = "/var/spool/cron";  // UserCrontabEntry
static SYSTEM_CRONTAB_DIR: &'static str = "/etc/cron.d";  // SystemCrontabEntry
static ANACRONTAB_FILE: &'static str = "/etc/anacrontab";  // AnacrontabEntry

fn process_crontab_file<T: ToCrontabEntry, P: AsRef<Path>>(path: P) {
    let crontab: CrontabFile<T> = CrontabFile::new(path.as_ref()).unwrap();
    for entry in crontab {
        match entry {
            Ok(data) => generate_systemd_units(path.as_ref(), data),
            Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Io(_), .. }) => panic!("error parsing file {}: {}", path.as_ref().display(), err),
            Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Parse(_), .. }) => println!("skipping file {} due to parsing error: {}", path.as_ref().display(), err),
        }
    }
}

fn process_crontab_dir<T: ToCrontabEntry>(dir: &str) {
    for fres in walk_dir(dir).unwrap() {
        let path = fres.unwrap().path();
        if !path.is_file() {
            continue;
        }

        process_crontab_file::<T, _>(path);
    }
}

fn main() {
    let user_thread = spawn(|| process_crontab_dir::<UserCrontabEntry>(USERS_CRONTAB_DIR));
    let system_thread = spawn(|| process_crontab_dir::<SystemCrontabEntry>(SYSTEM_CRONTAB_DIR));
    let anacron_thread = spawn(|| process_crontab_file::<AnacrontabEntry, _>(ANACRONTAB_FILE));

    user_thread.join();
    system_thread.join();
    anacron_thread.join();
}
