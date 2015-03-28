#![feature(core)]
#![feature(convert)]
#![feature(std_misc)]

use std::ops::Add;
use std::fs::File;
use std::io::{self, Lines, BufReader, BufRead};
use std::iter::Iterator;
use std::error::FromError;
use std::convert::AsRef;
use std::path::Path;

pub trait Limited: Add<u8, Output=Self> + PartialOrd + Copy {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

pub mod interval;
pub mod schedule;
pub mod crontab;

pub struct CrontabFile<T: crontab::ToCrontabEntry> {
    lines: Lines<BufReader<File>>,
    _marker: std::marker::PhantomData<T>
}

impl<T: crontab::ToCrontabEntry> CrontabFile<T> {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<CrontabFile<T>> {
        File::open(path).map(CrontabFile::from_file)
    }

    pub fn from_file(file: File) -> CrontabFile<T> {
        CrontabFile {
            lines: BufReader::new(file).lines(),
            _marker: std::marker::PhantomData
        }
    }
}

#[derive(Debug)]
pub enum CrontabFileError {
    Io(io::Error),
    Parse(crontab::CrontabEntryParseError)
}

impl FromError<io::Error> for CrontabFileError {
    fn from_error(err: io::Error) -> CrontabFileError {
        CrontabFileError::Io(err)
    }
}

impl FromError<crontab::CrontabEntryParseError> for CrontabFileError {
    fn from_error(err: crontab::CrontabEntryParseError) -> CrontabFileError {
        CrontabFileError::Parse(err)
    }
}


impl<T: crontab::ToCrontabEntry> Iterator for CrontabFile<T> {
    type Item = Result<crontab::CrontabEntry, CrontabFileError>;
    fn next(&mut self) -> Option<Result<crontab::CrontabEntry, CrontabFileError>> {
        loop {
            match self.lines.next() {
                Some(Ok(line)) => {
                    if line.len() == 0 || line.starts_with("#") || line.chars().all(|c| c == ' ' || c == '\t') {
                        continue;
                    }

                    return Some(match line.parse::<crontab::EnvVarEntry>() {
                        Ok(envvar) => Ok(crontab::CrontabEntry::EnvVar(envvar)),
                        _ => line.parse::<T>().map_err(FromError::from_error).map(crontab::ToCrontabEntry::to_crontab_entry)
                    });
                },
                Some(Err(e)) => return Some(Err(FromError::from_error(e))),
                None => return None
            }
        }
    }
}

