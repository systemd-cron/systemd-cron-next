#![allow(dead_code)] // for debug

extern crate cronparse;

use cronparse::interval::Interval;
use cronparse::schedule::DayOfWeek;
use cronparse::crontab::RootCrontabEntry;

fn main() {
    let s : Vec<Interval<DayOfWeek>> = "mon,sun-sat/2,*".parse().unwrap();
    println!("{}", s[1]);
    for v in s.iter().flat_map(Interval::iter) {
        println!("{:?}", v);
    }

    println!("{:?}", "* b c\td\t e  ".splitn(6, &[' ', '\t'][..]).collect::<Vec<&str>>());

    println!("{:?}", "* * * * * root command with args".parse::<RootCrontabEntry>());
}
