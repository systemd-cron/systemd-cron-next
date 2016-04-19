extern crate cronparse;
extern crate libc;
extern crate md5;
extern crate pgs_files;

#[macro_use]
extern crate log;
extern crate kernlog;

use std::thread::spawn;
use std::env;
use std::fs::{File, create_dir_all, metadata};
use std::os::unix::fs::symlink;
use std::io::Write;
use std::path::Path;

use cronparse::crontab::{AnacrontabEntry, SystemCrontabEntry, UserCrontabEntry};

mod generate;
mod process;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
static SYSTEM_CRONTAB_DIR: &'static str = "/etc/cron.d";  // SystemCrontabEntry
static SYSTEM_CRONTAB_FILE: &'static str = "/etc/crontab";
static ANACRONTAB_FILE: &'static str = "/etc/anacrontab";  // AnacrontabEntry
static REBOOT_FILE: &'static str = "/run/crond.reboot";

macro_rules! try_ {
    ($exp:expr) => {
        match $exp {
            Ok(value) => value,
            Err(err) => { error!("{}", err); return; }
        }
    }
}

fn main() {
    log::set_logger(|filter| kernlog::KernelLog::init_level(log::LogLevelFilter::Error, filter)).unwrap();

    let dest_dir = match env::args().nth(1) {
        None => {
            println!("Usage: systemd-crontab-generator <destination-directory>");
            return;
        }
        Some(path) => path,
    };

    let s = dest_dir.clone();
    let user_thread = spawn(move || {
        if !metadata(USERS_CRONTAB_DIR).map(|m| m.is_dir()).unwrap_or(false) {
            return generate_after_var_unit(&*s);
        }

        process::process_crontab_dir::<UserCrontabEntry, _>(USERS_CRONTAB_DIR, s);
        create_reboot_lock_file();
    });

    let s = dest_dir.clone();
    let system_thread = spawn(move || {
        process::process_crontab_file::<SystemCrontabEntry, _, _>(SYSTEM_CRONTAB_FILE, &s);
        process::process_crontab_dir::<SystemCrontabEntry, _>(SYSTEM_CRONTAB_DIR, &s);
    });

    let s = dest_dir.clone();
    let anacron_thread = spawn(move || {
        process::process_crontab_file::<AnacrontabEntry, _, _>(ANACRONTAB_FILE, &s);
    });

    let _ = user_thread.join();
    let _ = system_thread.join();
    let _ = anacron_thread.join();
}

fn generate_after_var_unit(dest_dir: &str) {
    let cron_after_var_unit_path = Path::new(dest_dir).join("cron-after-var.service");
    let mut cron_after_var_unit_file = try_!(File::create(&cron_after_var_unit_path));
    try_!(writeln!(cron_after_var_unit_file,
                   r###"[Unit]
Description=Rerun systemd-crontab-generator because /var is a separate mount
Documentation=man:systemd.cron(7)
After=cron.target
ConditionDirectoryNotEmpty={statedir}

[Service]
Type=oneshot
ExecStart=/bin/sh -c "{bindir}/systemctl daemon-reload ; {bindir}/systemctl try-restart cron.target""###,
                   statedir = USERS_CRONTAB_DIR,
                   bindir = BIN_DIR));

    let multiuser_wants_path = Path::new(dest_dir).join("multi-user.target.wants");
    try_!(create_dir_all(&multiuser_wants_path));
    try_!(symlink(cron_after_var_unit_path, multiuser_wants_path.join("cron-after-var.service")));
}

fn create_reboot_lock_file() {
    if let Err(err) = File::create(REBOOT_FILE) {
        warn!("error creating lock file {}: {}", REBOOT_FILE, err);
    }
}
