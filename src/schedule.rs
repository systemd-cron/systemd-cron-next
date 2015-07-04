#![allow(unused_comparisons)]

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::ops::Add;
use std::num::{FromPrimitive, ParseIntError};
use std::ascii::AsciiExt;
use std::iter::{FromIterator, IntoIterator, Iterator};
use std::error::Error;
use std::convert::From;

use interval::{Interval, IntervalParseError};
use super::Limited;

#[derive(Debug, PartialEq)]
pub enum Schedule {
    Calendar(Calendar),
    Period(Period)
}

#[derive(Debug, PartialEq)]
pub enum Period {
    Reboot,
    Minutely,
    Hourly,
    Midnight,
    Daily,
    Weekly,
    Monthly,
    Quaterly,
    Biannually,
    Yearly,
    Days(u16),
}

#[derive(Debug, PartialEq)]
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

macro_rules! parse_cron_rec_field {
    ($iter:expr, $miss:ident, $err:ident) => {
        (match $iter.next().map(|s| s.parse().map_err(CalendarParseError::$err)).unwrap_or(Err(CalendarParseError::$miss)) {
            Err(e) => return Err(e),
            Ok(v) => v
        })
    };
}

#[derive(Debug, PartialEq)]
pub enum CalendarParseError {
    Minutes(IntervalParseError),
    Hours(IntervalParseError),
    Days(IntervalParseError),
    Months(IntervalParseError),
    DaysOfWeek(IntervalParseError),
    MissingMinutes,
    MissingHours,
    MissingDays,
    MissingMonths,
    MissingDaysOfWeek
}

impl Display for CalendarParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::CalendarParseError::*;
        match *self {
            Minutes(ref e) => write!(f, "invalid minutes: {}", e),
            Hours(ref e) => write!(f, "invalid hours: {}", e),
            Days(ref e) => write!(f, "invalid days: {}", e),
            Months(ref e) => write!(f, "invalid months: {}", e),
            DaysOfWeek(ref e) => write!(f, "invalid days of week: {}", e),
            MissingMinutes => f.write_str("missing minutes"),
            MissingHours => f.write_str("missing hours"),
            MissingDays => f.write_str("missing days"),
            MissingMonths => f.write_str("missing months"),
            MissingDaysOfWeek => f.write_str("missing days of week"),
        }
    }
}

impl Error for CalendarParseError {
    fn description(&self) -> &str {
        use self::CalendarParseError::*;
        match *self {
            Minutes(_) => "invalid minutes",
            Hours(_) => "invalid hours",
            Days(_) => "invalid days",
            Months(_) => "invalid months",
            DaysOfWeek(_) => "invalid days of week",
            MissingMinutes => "missing minutes",
            MissingHours => "missing hours",
            MissingDays => "missing days",
            MissingMonths => "missing months",
            MissingDaysOfWeek => "missing days of week",
        }
    }
    fn cause(&self) -> Option<&Error> {
        use self::CalendarParseError::*;
        match *self {
            Minutes(ref e) | Hours(ref e) | Days(ref e) | Months(ref e) | DaysOfWeek(ref e) => Some(e),
            _ => None
        }
    }
}

impl FromStr for Calendar {
    type Err = CalendarParseError;
    fn from_str(s: &str) -> Result<Calendar, CalendarParseError> {
        let seps = [' ', '\t'];
        s.split(&seps[..]).filter(|v| *v != "").collect::<Result<Calendar, CalendarParseError>>()
    }
}

impl<'a> FromIterator<&'a str> for Result<Calendar, CalendarParseError> {
    fn from_iter<T>(iter: T) -> Result<Calendar, CalendarParseError> where T: IntoIterator<Item=&'a str> {
        let mut it = iter.into_iter();
        Ok(Calendar {
            mins: parse_cron_rec_field!(it, MissingMinutes, Minutes),
            hrs: parse_cron_rec_field!(it, MissingHours, Hours),
            days: parse_cron_rec_field!(it, MissingDays, Days),
            mons: parse_cron_rec_field!(it, MissingMonths, Months),
            dows: parse_cron_rec_field!(it, MissingDaysOfWeek, DaysOfWeek)
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum PeriodParseError {
    InvalidDays(ParseIntError),
    UnknownPeriod
}

impl From<ParseIntError> for PeriodParseError {
    fn from(e: ParseIntError) -> PeriodParseError {
        PeriodParseError::InvalidDays(e)
    }
}

impl Display for PeriodParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            PeriodParseError::InvalidDays(ref e) => e.fmt(f),
            PeriodParseError::UnknownPeriod => f.write_str("unknown period name")
        }
    }
}

impl Error for PeriodParseError {
    fn description(&self) -> &str {
        match *self {
            PeriodParseError::InvalidDays(_) => "invalid days value",
            PeriodParseError::UnknownPeriod => "unknown period name",
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            PeriodParseError::InvalidDays(ref e) => Some(e),
            PeriodParseError::UnknownPeriod => None
        }
    }
}

impl FromStr for Period {
    type Err = PeriodParseError;
    fn from_str(s: &str) -> Result<Period, PeriodParseError> {
        Ok(match s {
            "@reboot" => Period::Reboot,
            "@minutely" => Period::Minutely,
            "@hourly" => Period::Hourly,
            "@midnight" => Period::Midnight,
            "@daily" | "1" => Period::Daily,
            "@weekly" | "7" => Period::Weekly,
            "@monthly" | "30" | "31" => Period::Monthly,
            "@quaterly" => Period::Quaterly,
            "@biannually" | "@bi-annually" | "@semiannually" => Period::Biannually,
            "@yearly" | "@annually" | "@anually" => Period::Yearly,
            r @ _ if r.starts_with("@") => return Err(PeriodParseError::UnknownPeriod),
            _ => try!(s.parse().map(Period::Days)),
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
            type Err = ParseIntError;
            #[inline]
            fn from_str(s: &str) -> Result<$name, <u8 as FromStr>::Err> {
                s.parse().map($name)
            }
        }
    }
}

#[derive(Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Minute(pub u8);
limited!(Minute, min=0, max=59);

#[derive(Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hour(pub u8);
limited!(Hour, min=0, max=23);

#[derive(Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day(pub u8);
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

#[derive(Debug, PartialEq)]
pub struct MonthParseError;

impl Error for MonthParseError {
    fn cause(&self) -> Option<&Error> { None }
    fn description(&self) -> &str { "invalid month" }
}

impl Display for MonthParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("invalid month")
    }
}

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

#[derive(Debug, PartialEq)]
pub struct DayOfWeekParseError;

impl Error for DayOfWeekParseError {
    fn cause(&self) -> Option<&Error> { None }
    fn description(&self) -> &str { "invalid day of week" }
}

impl Display for DayOfWeekParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("invalid day of week")
    }
}

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

#[derive(Debug, PartialEq)]
pub enum ScheduleParseError {
    MissingSchedule,
    InvalidPeriod(PeriodParseError),
    InvalidCalendar(CalendarParseError)
}

impl From<PeriodParseError> for ScheduleParseError {
    fn from(e: PeriodParseError) -> ScheduleParseError {
        ScheduleParseError::InvalidPeriod(e)
    }
}

impl From<CalendarParseError> for ScheduleParseError {
    fn from(e: CalendarParseError) -> ScheduleParseError {
        ScheduleParseError::InvalidCalendar(e)
    }
}

impl Error for ScheduleParseError {
    fn description(&self) -> &str {
        match *self {
            ScheduleParseError::InvalidPeriod(_) => "invalid period",
            ScheduleParseError::InvalidCalendar(_) => "invalid calendar",
            ScheduleParseError::MissingSchedule => "missing schedule",
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            ScheduleParseError::InvalidPeriod(ref e) => Some(e),
            ScheduleParseError::InvalidCalendar(ref e) => Some(e),
            _ => None,
        }
    }
}

impl Display for ScheduleParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ScheduleParseError::InvalidPeriod(ref e) => e.fmt(f),
            ScheduleParseError::InvalidCalendar(ref e) => e.fmt(f),
            ScheduleParseError::MissingSchedule => f.write_str("missing schedule"),
        }
    }
}

impl FromStr for Schedule {
    type Err = ScheduleParseError;
    fn from_str(s: &str) -> Result<Schedule, ScheduleParseError> {
        if s.starts_with("@") {
            s.parse::<Period>().map_err(From::from).map(Schedule::Period)
        } else {
            s.parse::<Calendar>().map_err(From::from).map(Schedule::Calendar)
        }
    }
}

impl<'a> FromIterator<&'a str> for Result<Schedule, ScheduleParseError> {
    fn from_iter<T>(iter: T) -> Result<Schedule, ScheduleParseError> where T: IntoIterator<Item=&'a str> {
        let mut it = iter.into_iter().peekable();
        let is_period = match it.peek() {
            None => return Err(ScheduleParseError::MissingSchedule),
            Some(s) => s.starts_with("@")
        };

        if is_period {
            it.next().map(|p| p.parse().map_err(ScheduleParseError::InvalidPeriod).map(Schedule::Period)).unwrap_or(Err(ScheduleParseError::MissingSchedule))
        } else {
            it.collect::<Result<Calendar, CalendarParseError>>().map_err(From::from).map(Schedule::Calendar)
        }
    }
}


