use std::str::FromStr;
use std::fmt::{self, Display, Formatter};

use super::Limited;

#[derive(Debug)]
pub enum Interval<T: Limited> {
    Value(T),
    Range(T, T, u8),
    Full(u8)
}

#[derive(Debug)]
pub struct IntervalParseError;

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

impl<T: Limited + FromStr> FromStr for Vec<Interval<T>> {
    type Err = IntervalParseError;
    fn from_str(s: &str) -> Result<Vec<Interval<T>>, IntervalParseError> {
        s.split(',').map(|v| v.parse::<Interval<T>>()).collect()
    }
}
