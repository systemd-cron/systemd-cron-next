#![feature(core)]
#![feature(convert)]
#![feature(std_misc)]

use std::ops::{Add, Deref};
use std::fs::File;
use std::io::{self, Lines, BufReader, BufRead};
use std::iter::{Iterator, Enumerate};
use std::error::{FromError, Error};
use std::convert::AsRef;
use std::path::Path;
use std::fmt::{self, Display, Formatter};

pub trait Limited: Add<u8, Output=Self> + Ord + Copy {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

#[macro_use]
pub mod interval;
pub mod schedule;
pub mod crontab;

pub struct CrontabFile<T: crontab::ToCrontabEntry> {
    lines: Enumerate<Lines<BufReader<File>>>,
    _marker: std::marker::PhantomData<T>
}

impl<T: crontab::ToCrontabEntry> CrontabFile<T> {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<CrontabFile<T>> {
        File::open(path).map(CrontabFile::from_file)
    }

    pub fn from_file(file: File) -> CrontabFile<T> {
        CrontabFile {
            lines: BufReader::new(file).lines().enumerate(),
            _marker: std::marker::PhantomData
        }
    }
}

#[derive(Debug)]
pub enum CrontabFileErrorKind {
    Io(io::Error),
    Parse(crontab::CrontabEntryParseError)
}

impl Display for CrontabFileErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CrontabFileErrorKind::Io(ref e) => e.fmt(f),
            CrontabFileErrorKind::Parse(ref e) => e.fmt(f)
        }
    }
}

#[derive(Debug)]
pub struct CrontabFileError {
    pub lineno: usize,
    pub line: Option<String>,
    pub kind: CrontabFileErrorKind
}

impl FromError<io::Error> for CrontabFileError {
    fn from_error(err: io::Error) -> CrontabFileError {
        CrontabFileError {
            lineno: 0,
            line: None,
            kind: CrontabFileErrorKind::Io(err)
        }
    }
}

impl FromError<crontab::CrontabEntryParseError> for CrontabFileError {
    fn from_error(err: crontab::CrontabEntryParseError) -> CrontabFileError {
        CrontabFileError {
            lineno: 0,
            line: None,
            kind: CrontabFileErrorKind::Parse(err)
        }
    }
}

impl Error for CrontabFileError {
    fn description(&self) -> &str {
        "error parsing crontab"
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            CrontabFileErrorKind::Parse(ref e) => Some(e),
            CrontabFileErrorKind::Io(ref e) => Some(e)
        }
    }
}

impl Display for CrontabFileError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "error parsing crontab at line {} ({:?}): {}", self.lineno, self.line.as_ref().map(Deref::deref).unwrap_or("<???>"), self.kind)
    }
}

impl<T: crontab::ToCrontabEntry> Iterator for CrontabFile<T> {
    type Item = Result<crontab::CrontabEntry, CrontabFileError>;
    fn next(&mut self) -> Option<Result<crontab::CrontabEntry, CrontabFileError>> {
        loop {
            match self.lines.next() {
                Some((lineno, Ok(line))) => {
                    if line.len() == 0 || line.starts_with("#") || line.chars().all(|c| c == ' ' || c == '\t') {
                        continue;
                    }

                    return Some(match line.parse::<crontab::EnvVarEntry>() {
                        Ok(envvar) => Ok(crontab::CrontabEntry::EnvVar(envvar)),
                        _ => line.parse::<T>().map_err(|e| {
                            let mut err: CrontabFileError = FromError::from_error(e);
                            err.lineno = lineno + 1;
                            err.line = Some(line.to_string());
                            err
                        }).map(crontab::ToCrontabEntry::to_crontab_entry)
                    });
                },
                Some((lineno, Err(e))) => {
                    let mut err: CrontabFileError = FromError::from_error(e);
                    err.lineno = lineno + 1;
                    return Some(Err(err));
                },
                None => return None
            }
        }
    }
}

