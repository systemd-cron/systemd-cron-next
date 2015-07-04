use std::str::FromStr;
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use std::convert::From;
use std::num::ParseIntError;

use super::Limited;
use schedule::{MonthParseError, DayOfWeekParseError};
use self::IntervalParseError::*;

#[derive(Debug, PartialEq)]
pub enum Interval<T: Limited> {
    Value(T),
    Range(T, T, u8),
    Full(u8)
}

#[derive(Debug, PartialEq)]
pub struct Intervals<T: Limited>(pub Vec<Interval<T>>);

#[derive(Debug, PartialEq)]
pub enum IntervalParseError {
    ZeroStep,
    InvalidInteger(ParseIntError),
    InvalidMonth(MonthParseError),
    InvalidDayOfWeek(DayOfWeekParseError),
    InverseRange
}

impl From<ParseIntError> for IntervalParseError {
    fn from(err: ParseIntError) -> IntervalParseError {
        IntervalParseError::InvalidInteger(err)
    }
}

impl From<MonthParseError> for IntervalParseError {
    fn from(err: MonthParseError) -> IntervalParseError {
        IntervalParseError::InvalidMonth(err)
    }
}

impl From<DayOfWeekParseError> for IntervalParseError {
    fn from(err: DayOfWeekParseError) -> IntervalParseError {
        IntervalParseError::InvalidDayOfWeek(err)
    }
}

impl Display for IntervalParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            InverseRange => f.write_str("range end is less than start"),
            ZeroStep => f.write_str("step is zero"),
            InvalidInteger(ref e) => e.fmt(f),
            InvalidMonth(ref e) => e.fmt(f),
            InvalidDayOfWeek(ref e) => e.fmt(f),
        }
    }
}

impl Error for IntervalParseError {
    fn description(&self) -> &str {
        match *self {
            InvalidInteger(_) => "invalid integer format",
            InvalidMonth(_) => "invalid month value",
            InvalidDayOfWeek(_) => "invalid day of week value",
            ZeroStep => "step is zero",
            InverseRange => "range end is less than start"
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            InvalidInteger(ref e) => Some(e),
            InvalidMonth(ref e) => Some(e),
            InvalidDayOfWeek(ref e) => Some(e),
            _ => None
        }
    }
}

impl<T> FromStr for Interval<T>
where T: Limited, T: FromStr, IntervalParseError: From<<T as FromStr>::Err>
{
    type Err = IntervalParseError;
    fn from_str(s: &str) -> Result<Interval<T>, IntervalParseError> {
        let (range, step): (&str, u8) = if let Some(slash) = s.find('/') {
            (&s[..slash], try!(s[slash+1..].parse())) // FIXME
        } else {
            (s, 1)
        };

        if step == 0 {
            return Err(IntervalParseError::ZeroStep)
        }

        if range == "*" {
            return Ok(Interval::Full(step))
        }

        let (from, to): (T, T) = if let Some(hyphen) = range.find('-') {
            (try!(range[..hyphen].parse()), try!(range[hyphen+1..].parse()))
        } else {
            return range.parse().map_err(From::from).map(Interval::Value);
        };

        if from > to {
            return Err(IntervalParseError::InverseRange)
        }

        Ok(Interval::Range(from, to, step))
    }
}

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

impl<T> Interval<T> where T: Limited {
    pub fn new(val: T) -> Interval<T> {
        Interval::Value(val)
    }

    pub fn full_step(step: u8) -> Interval<T> {
        Interval::Full(step)
    }

    pub fn full() -> Interval<T> {
        Interval::Full(1)
    }

    pub fn from_range(from: T, to: T) -> Interval<T> {
        Interval::Range(from, to, 1)
    }

    pub fn from_range_step(from: T, to: T, step: u8) -> Interval<T> {
        Interval::Range(from, to, step)
    }

    pub fn iter(&self) -> IntervalIter<T> {
        match *self {
            Interval::Value(v) => IntervalIter { cur: v, max: v, step: 1, stop: false },
            Interval::Range(f, t, s) => IntervalIter { cur: f, max: t, step: s, stop: false },
            Interval::Full(s) => IntervalIter { cur: T::min_value(), max: T::max_value(), step: s, stop: false }
        }
    }
}

#[derive(Debug)]
pub struct IntervalIter<T: Limited> {
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

impl<T: Limited + FromStr> FromStr for Intervals<T>
where T: Limited, T: FromStr, IntervalParseError: From<<T as FromStr>::Err>
{
    type Err = IntervalParseError;
    fn from_str(s: &str) -> Result<Intervals<T>, IntervalParseError> {
        s.split(',').map(|v| v.parse::<Interval<T>>()).collect::<Result<Vec<_>, _>>().map(Intervals)
    }
}
