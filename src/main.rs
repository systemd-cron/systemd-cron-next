#![feature(fs_walk)]
#![feature(path_ext)]
#![feature(libc)]
#![feature(slice_concat_ext)]

extern crate cronparse;
extern crate libc;
extern crate md5;

#[macro_use]
extern crate log;
extern crate kernlog;

use std::path::Path;
use std::thread::spawn;
use std::env;

use cronparse::{CrontabFile, CrontabFileError, CrontabFileErrorKind};
use cronparse::crontab::{UserCrontabEntry, SystemCrontabEntry, AnacrontabEntry, CrontabEntry, ToCrontabEntry};

mod process;

static USERS_CRONTAB_DIR: &'static str = "/var/spool/cron";  // UserCrontabEntry
static SYSTEM_CRONTAB_DIR: &'static str = "/etc/cron.d";  // SystemCrontabEntry
static ANACRONTAB_FILE: &'static str = "/etc/anacrontab";  // AnacrontabEntry

#[inline]
fn is_run_by_systemd() -> bool {
    env::args().len() >= 3
}

fn main() {
    let dest_dir = match env::args().next() {
        None => {
            println!("Usage: systemd-crontab-generator <destination-directory>");
            return;
        },
        Some(path) => path
    };

    let s = dest_dir.clone();
    let user_thread = spawn(|| process::process_crontab_dir::<UserCrontabEntry, _>(USERS_CRONTAB_DIR, s));

    let s = dest_dir.clone();
    let system_thread = spawn(|| process::process_crontab_dir::<SystemCrontabEntry, _>(SYSTEM_CRONTAB_DIR, s));

    let s = dest_dir.clone();
    let anacron_thread = spawn(|| process::process_crontab_file::<AnacrontabEntry, _, _>(ANACRONTAB_FILE, s));

    let _ = user_thread.join();
    let _ = system_thread.join();
    let _ = anacron_thread.join();
}
