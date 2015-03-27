#![feature(core)]
#![allow(dead_code)] // for debug

use std::ops::Add;
use std::num::FromPrimitive;
use std::iter::{Iterator, IteratorExt};
use std::str::FromStr;
use std::ascii::AsciiExt;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
struct CrontabRecord {
    mins: Minutes,
    hrs: Hours,
    days: Days,
    mons: Months,
    dows: DaysOfWeek,
    user: Option<String>,
    cmd: String
}

#[derive(Debug)]
struct CrontabRecordParseError;

macro_rules! parse_cron_rec_field {
    ($iter:expr) => {
        try!($iter.next().ok_or(IntervalParseError).and_then(FromStr::from_str).map_err(|_| CrontabRecordParseError))
    }
}

impl FromStr for CrontabRecord {
    type Err = CrontabRecordParseError;

    fn from_str(s: &str) -> Result<CrontabRecord, CrontabRecordParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]);
        Ok(CrontabRecord {
            mins: parse_cron_rec_field!(splits),
            hrs: parse_cron_rec_field!(splits),
            days: parse_cron_rec_field!(splits),
            mons: parse_cron_rec_field!(splits),
            dows: parse_cron_rec_field!(splits),
            user: None,
            cmd: splits.collect::<Vec<&str>>().connect(" ")
        })
    }
}

type Minutes = Vec<Interval<Minute>>;
type Hours = Vec<Interval<Hour>>;
type Days = Vec<Interval<Day>>;
type Months = Vec<Interval<Month>>;
type DaysOfWeek = Vec<Interval<DayOfWeek>>;

#[derive(Debug)]
enum Interval<T: Limited> {
    Value(T),
    Range(T, T, u8),
    Full(u8)
}

#[derive(Debug)]
struct IntervalParseError;

impl<T: Limited + Display> Display for Interval<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Interval::Value(val) => val.fmt(f),
            Interval::Range(from, to, step) => from.fmt(f)
                .and_then(|_| f.write_str("-"))
                .and_then(|_| to.fmt(f))
                .and_then(|_| if step > 1 {
                    f.write_str("/").and_then(|_| step.fmt(f))
                } else {
                    Ok(())
                }),
            Interval::Full(step) => f.write_str("*")
                .and_then(|_| if step > 1 {
                    f.write_str("/").and_then(|_| step.fmt(f))
                } else {
                    Ok(())
                })
        }
    }
}

impl<T: Limited + FromStr> FromStr for Interval<T> {
    type Err = IntervalParseError;
    fn from_str(s: &str) -> Result<Interval<T>, IntervalParseError> {
        let (range, step): (&str, u8) = if let Some(slash) = s.find('/') {
            (&s[..slash], try!(s[slash+1..].parse().map_err(|_| IntervalParseError))) // FIXME
        } else {
            (s, 1)
        };

        if step == 0 {
            return Err(IntervalParseError)
        }

        if range == "*" {
            return Ok(Interval::Full(step))
        }

        let (from, to): (T, T) = if let Some(hyphen) = range.find('-') {
            (try!(range[..hyphen].parse().map_err(|_| IntervalParseError)),
            try!(range[hyphen+1..].parse().map_err(|_| IntervalParseError)))
        } else {
            return range.parse().map_err(|_| IntervalParseError).map(Interval::Value);
        };

        if from > to {
            return Err(IntervalParseError)
        }

        Ok(Interval::Range(from, to, step))
    }
}

impl<T> Interval<T> where T: Limited {
    fn new(val: T) -> Interval<T> {
        Interval::Value(val)
    }

    fn full_step(step: u8) -> Interval<T> {
        Interval::Full(step)
    }

    fn full() -> Interval<T> {
        Interval::Full(1)
    }

    fn from_range(from: T, to: T) -> Interval<T> {
        Interval::Range(from, to, 1)
    }

    fn from_range_step(from: T, to: T, step: u8) -> Interval<T> {
        Interval::Range(from, to, step)
    }

    fn iter(&self) -> IntervalIter<T> {
        match *self {
            Interval::Value(v) => IntervalIter { cur: v, max: v, step: 1, stop: false },
            Interval::Range(f, t, s) => IntervalIter { cur: f, max: t, step: s, stop: false },
            Interval::Full(s) => IntervalIter { cur: T::min_value(), max: T::max_value(), step: s, stop: false }
        }
    }
}

#[derive(Debug)]
struct IntervalIter<T: Limited> {
    cur: T,
    max: T,
    step: u8,
    stop: bool
}

impl<T: Limited> Iterator for IntervalIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.stop {
            return None;
        }

        if self.cur <= self.max {
            let result = self.cur;
            self.cur = self.cur + self.step;
            self.stop = result >= self.max;
            Some(result)
        } else {
            self.stop = true;
            None
        }
    }
}

trait Limited: Add<u8, Output=Self> + PartialOrd + Copy {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

macro_rules! limited {
    ($name:ident, min=$min:expr, max=$max:expr) => {
        impl Limited for $name {
            fn min_value() -> $name { $name($min) }
            fn max_value() -> $name { $name($max) }
        }

        impl Add<u8> for $name {
            type Output = $name;
            fn add(self, rhs: u8) -> $name {
                //$name(((self.0 - $min) + rhs) % ($max - $min) + $min)
                let val = self.0 + rhs;
                $name(if val < $min { $min } else if val > $max { $max } else { val })
            }
        }

        impl Display for $name {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = <u8 as FromStr>::Err;
            #[inline]
            fn from_str(s: &str) -> Result<$name, <u8 as FromStr>::Err> {
                s.parse().map($name)
            }
        }
    }
}

#[derive(Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Minute(u8);
limited!(Minute, min=0, max=59);

#[derive(Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Hour(u8);
limited!(Hour, min=0, max=23);

#[derive(Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Day(u8);
limited!(Day, min=1, max=31);

#[derive(Debug, Copy, FromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12
}

#[derive(Debug)]
struct MonthParseError;

impl FromStr for Month {
    type Err = MonthParseError;
    fn from_str(s: &str) -> Result<Month, MonthParseError> {
        s.parse::<u8>()
            .map_err(|_| MonthParseError)
            .and_then(|v| Month::from_u8(v).ok_or(MonthParseError))
            .or_else(|_| match &*s[..3].to_ascii_lowercase() {
                "jan" => Ok(Month::January),
                "feb" => Ok(Month::February),
                "mar" => Ok(Month::March),
                "apr" => Ok(Month::April),
                "may" => Ok(Month::May),
                "jun" => Ok(Month::June),
                "jul" => Ok(Month::July),
                "aug" => Ok(Month::August),
                "sep" => Ok(Month::September),
                "oct" => Ok(Month::October),
                "nov" => Ok(Month::November),
                "dec" => Ok(Month::December),
                _ => Err(MonthParseError)
            })
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match *self {
            Month::January => "Jan",
            Month::February => "Feb",
            Month::March => "Mar",
            Month::April => "Apr",
            Month::May => "May",
            Month::June => "Jun",
            Month::July => "Jul",
            Month::August => "Aug",
            Month::September => "Sep",
            Month::October => "Oct",
            Month::November => "Nov",
            Month::December => "Dec"
        })
    }
}

impl Limited for Month {
    fn min_value() -> Month { Month::January }
    fn max_value() -> Month { Month::December }
}

impl Add<u8> for Month {
    type Output = Month;
    fn add(self, rhs: u8) -> Month {
        let val = self as u8 + rhs;
        if val < 1 { Month::January }
        else if val > 12 { Month::December }
        else { FromPrimitive::from_u8(val).unwrap() }
    }
}

#[derive(Debug, Copy, FromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum DayOfWeek {
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}
impl Limited for DayOfWeek {
    fn min_value() -> DayOfWeek { DayOfWeek::Sunday }
    fn max_value() -> DayOfWeek { DayOfWeek::Saturday }
}

impl Display for DayOfWeek {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match *self {
            DayOfWeek::Sunday => "Sun",
            DayOfWeek::Monday => "Mon",
            DayOfWeek::Tuesday => "Tue",
            DayOfWeek::Wednesday => "Wed",
            DayOfWeek::Thursday => "Thu",
            DayOfWeek::Friday => "Fri",
            DayOfWeek::Saturday => "Sat",
        })
    }
}

#[derive(Debug)]
struct DayOfWeekParseError;

impl FromStr for DayOfWeek {
    type Err = DayOfWeekParseError;
    fn from_str(s: &str) -> Result<DayOfWeek, DayOfWeekParseError> {
        s.parse::<u8>()
            .map_err(|_| DayOfWeekParseError)
            .and_then(|v| DayOfWeek::from_u8(v % 7).ok_or(DayOfWeekParseError))
            .or_else(|_| match &*s[..3].to_ascii_lowercase() {
                "sun" => Ok(DayOfWeek::Sunday),
                "mon" => Ok(DayOfWeek::Monday),
                "tue" => Ok(DayOfWeek::Tuesday),
                "wed" => Ok(DayOfWeek::Wednesday),
                "thu" => Ok(DayOfWeek::Thursday),
                "fri" => Ok(DayOfWeek::Friday),
                "sat" => Ok(DayOfWeek::Saturday),
                _ => Err(DayOfWeekParseError)
            })
    }
}

impl Add<u8> for DayOfWeek {
    type Output = DayOfWeek;
    fn add(self, rhs: u8) -> DayOfWeek {
        let val = self as u8 + rhs;
        if val > 6 { DayOfWeek::Saturday }
        else { FromPrimitive::from_u8(val).unwrap() }
    }
}

impl<T: Limited + FromStr> FromStr for Vec<Interval<T>> {
    type Err = IntervalParseError;
    fn from_str(s: &str) -> Result<Vec<Interval<T>>, IntervalParseError> {
        s.split(',').map(|v| v.parse::<Interval<T>>()).collect()
    }
}

fn main() {
    let s : Vec<Interval<DayOfWeek>> = "mon,sun-sat/2,*".parse().unwrap();
    println!("{}", s[1]);
    for v in s.iter().flat_map(Interval::iter) {
        println!("{:?}", v);
    }

    println!("{:?}", "* b c\td\t e  ".splitn(6, &[' ', '\t'][..]).collect::<Vec<&str>>());

    println!("{:?}", "* * * * * root command with args".parse::<CrontabRecord>());
}
