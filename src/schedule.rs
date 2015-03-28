
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::ops::Add;
use std::num::FromPrimitive;
use std::ascii::AsciiExt;

use interval::Interval;
use super::Limited;

#[derive(Debug)]
pub enum Schedule {
    Calendar(Calendar),
    Period(Period)
}

#[derive(Debug)]
pub enum Period {
    Reboot,
    Minutely,
    Midnight,
    Daily,
    Weekly,
    Monthly,
    Quaterly,
    Biannually,
    Yearly,
    Other(String)
}

#[derive(Debug)]
pub struct Calendar {
    pub mins: Minutes,
    pub hrs: Hours,
    pub days: Days,
    pub mons: Months,
    pub dows: DaysOfWeek,
}

pub type Minutes = Vec<Interval<Minute>>;
pub type Hours = Vec<Interval<Hour>>;
pub type Days = Vec<Interval<Day>>;
pub type Months = Vec<Interval<Month>>;
pub type DaysOfWeek = Vec<Interval<DayOfWeek>>;

pub struct PeriodParseError;
impl FromStr for Period {
    type Err = PeriodParseError;
    fn from_str(s: &str) -> Result<Period, PeriodParseError> {
        Ok(match s {
            "@reboot" => Period::Reboot,
            "@minutely" => Period::Minutely,
            "@midnight" => Period::Midnight,
            "@daily" | "1" => Period::Daily,
            "@weekly" | "7" => Period::Weekly,
            "@monthly" | "30" | "31" => Period::Monthly,
            "@quaterly" => Period::Quaterly,
            "@biannually" | "@bi-annually" | "@semiannually" => Period::Biannually,
            "@yearly" | "@annually" | "@anually" => Period::Yearly,
            r @ _ if r.starts_with("@") => Period::Other(r[1..].to_string()),
            _ => return Err(PeriodParseError)
        })
    }
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
pub enum Month {
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
pub struct MonthParseError;

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
pub enum DayOfWeek {
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
pub struct DayOfWeekParseError;

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

