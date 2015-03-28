#![feature(core)]
#![feature(std_misc)]

use std::ops::Add;

pub trait Limited: Add<u8, Output=Self> + PartialOrd + Copy {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

pub mod interval;
pub mod schedule;
pub mod crontab;
