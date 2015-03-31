#![feature(path_ext)]
#![feature(fs_walk)]
#![feature(convert)]
#![feature(libc)]

extern crate cronparse;
extern crate libc;

use std::thread::spawn;
use std::env;

use cronparse::crontab::{UserCrontabEntry, SystemCrontabEntry, AnacrontabEntry};
use log::{KernelLogger, ConsoleLogger, AnyLogger};

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
    let user_thread = spawn(|| process::process_crontab_dir::<UserCrontabEntry>(USERS_CRONTAB_DIR, &mut get_logger()));
    let system_thread = spawn(|| process::process_crontab_dir::<SystemCrontabEntry>(SYSTEM_CRONTAB_DIR, &mut get_logger()));
    let anacron_thread = spawn(|| process::process_crontab_file::<AnacrontabEntry, _>(ANACRONTAB_FILE, &mut get_logger()));

    let _ = user_thread.join();
    let _ = system_thread.join();
    let _ = anacron_thread.join();
}
