#![allow(dead_code)] // for debug

extern crate cronparse;

use cronparse::CrontabFile;
use cronparse::crontab::{UserCrontabEntry, CrontabEntry};

fn generate_systemd_units(entry: CrontabEntry) {
    println!("{:?}", entry);
}

fn main() {
    let filename = "kstep";
    let file: CrontabFile<UserCrontabEntry> = CrontabFile::new(filename).unwrap();
    for line in file {
        match line {
            Ok(entry) => generate_systemd_units(entry),
            Err(error) => panic!("At file {}: {}", filename, error)
        }
    }
}
