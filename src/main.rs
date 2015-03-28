#![allow(dead_code)] // for debug

extern crate cronparse;

use cronparse::CrontabFile;
use cronparse::crontab::UserCrontabEntry;

fn main() {
    let file: CrontabFile<UserCrontabEntry> = CrontabFile::new("kstep").unwrap();
    for line in file {
        println!("{:?}", line);
    }
}
