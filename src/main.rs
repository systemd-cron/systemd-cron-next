#![feature(collections)]
#![feature(path_ext)]
#![feature(fs_walk)]
#![feature(convert)]
#![feature(libc)]
#![feature(core)]
#![feature(step_by)]
#![feature(rustc_private)]

extern crate serialize;
extern crate cronparse;
extern crate libc;

use std::path::Path;
use std::thread::spawn;
use std::env;

use cronparse::crontab::{UserCrontabEntry, SystemCrontabEntry, AnacrontabEntry};
use log::{KernelLogger, ConsoleLogger, AnyLogger};

mod md5;
mod log;
mod process;

static USERS_CRONTAB_DIR: &'static str = "/var/spool/cron";  // UserCrontabEntry
static SYSTEM_CRONTAB_DIR: &'static str = "/etc/cron.d";  // SystemCrontabEntry
static ANACRONTAB_FILE: &'static str = "/etc/anacrontab";  // AnacrontabEntry

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

fn main() {
    let dest_dir = match env::args().next() {
        None => {
            println!("Usage: systemd-crontab-generator <destination-directory>");
            return;
        },
        Some(path) => path
    };

    let s = dest_dir.clone();
    let user_thread = spawn(|| process::process_crontab_dir::<UserCrontabEntry, _>(USERS_CRONTAB_DIR, s, &mut get_logger()));

    let s = dest_dir.clone();
    let system_thread = spawn(|| process::process_crontab_dir::<SystemCrontabEntry, _>(SYSTEM_CRONTAB_DIR, s, &mut get_logger()));

    let s = dest_dir.clone();
    let anacron_thread = spawn(|| process::process_crontab_file::<AnacrontabEntry, _, _>(ANACRONTAB_FILE, s, &mut get_logger()));

    let _ = user_thread.join();
    let _ = system_thread.join();
    let _ = anacron_thread.join();
}
