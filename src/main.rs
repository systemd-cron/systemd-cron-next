#![feature(fs_walk)]
#![feature(path_ext)]
#![feature(libc)]

extern crate cronparse;
extern crate libc;
extern crate md5;
extern crate users;

#[macro_use]
extern crate log;
extern crate kernlog;

use std::thread::spawn;
use std::env;
use std::fs::{File, PathExt, create_dir_all};
use std::os::unix::fs::symlink;
use std::io::Write;
use std::path::Path;

use cronparse::crontab::{UserCrontabEntry, SystemCrontabEntry, AnacrontabEntry};

mod generate;
mod process;

static USERS_CRONTAB_DIR: &'static str = "/var/spool/cron";  // UserCrontabEntry
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
        },
        Some(path) => path
    };

    let s = dest_dir.clone();
    let user_thread = spawn(|| {
        if Path::new(USERS_CRONTAB_DIR).is_dir() {
            process::process_crontab_dir::<UserCrontabEntry, _>(USERS_CRONTAB_DIR, s);
            if let Err(err) = File::create(REBOOT_FILE) {
                warn!("error creating lock file {}: {}", REBOOT_FILE, err);
            }
        } else {
            let cron_after_var_unit_path = Path::new(&*s).join("cron-after-var.service");
            {
                let mut cron_after_var_unit_file = try_!(File::create(&cron_after_var_unit_path));
                try_!(writeln!(cron_after_var_unit_file, r###"[Unit]
Description=Rerun systemd-crontab-generator because /var is a separate mount
Documentation=man:systemd.cron(7)
After=cron.target
ConditionDirectoryNotEmpty={statedir}

[Service]
Type=oneshot
ExecStart=/bin/sh -c "{bindir}/systemctl daemon-reload ; {bindir}/systemctl try-restart cron.target""###,
                    statedir = USERS_CRONTAB_DIR,
                    bindir = "/usr/bin"));
            }

            let multiuser_wants_path = Path::new(&*s).join("multi-user.target.wants");
            try_!(create_dir_all(&multiuser_wants_path));
            try_!(symlink(cron_after_var_unit_path, multiuser_wants_path.join("cron-after-var.service")));
        }
    });

    let s = dest_dir.clone();
    let system_thread = spawn(|| {
        try_!(process::process_crontab_dir::<SystemCrontabEntry, _>(SYSTEM_CRONTAB_DIR, s));
        try_!(process::process_crontab_file::<SystemCrontabEntry, _, _>(SYSTEM_CRONTAB_FILE, s));
    });

    let s = dest_dir.clone();
    let anacron_thread = spawn(|| process::process_crontab_file::<AnacrontabEntry, _, _>(ANACRONTAB_FILE, s));

    let _ = user_thread.join();
    let _ = system_thread.join();
    let _ = anacron_thread.join();
}
