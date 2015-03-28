use std::time::Duration;
use std::str::FromStr;
use std::iter::FromIterator;

use schedule::{Schedule, Period, ScheduleParseError, PeriodParseError};

pub trait ToCrontabEntry : FromStr<Err=CrontabEntryParseError> {
    fn to_crontab_entry(self) -> CrontabEntry;
}

#[derive(Debug, PartialEq)]
pub enum CrontabEntry {
    EnvVar(EnvVarEntry),
    User(UserCrontabEntry),
    Root(RootCrontabEntry),
    Anacron(AnacrontabEntry)
}

#[derive(Debug, PartialEq)]
pub struct EnvVarEntry(pub String, pub String);

#[derive(Debug, PartialEq)]
pub struct UserCrontabEntry {
    pub sched: Schedule,
    pub cmd: String
}

#[derive(Debug, PartialEq)]
pub struct RootCrontabEntry {
    pub sched: Schedule,
    pub user: UserInfo,
    pub cmd: String
}

#[derive(Debug, PartialEq)]
pub struct AnacrontabEntry {
    pub period: Period,
    pub delay: Duration,
    pub jobid: String,
    pub cmd: String
}

impl ToCrontabEntry for UserCrontabEntry {
    fn to_crontab_entry(self) -> CrontabEntry {
        CrontabEntry::User(self)
    }
}

impl ToCrontabEntry for RootCrontabEntry {
    fn to_crontab_entry(self) -> CrontabEntry {
        CrontabEntry::Root(self)
    }
}

impl ToCrontabEntry for AnacrontabEntry {
    fn to_crontab_entry(self) -> CrontabEntry {
        CrontabEntry::Anacron(self)
    }
}

impl ToCrontabEntry for EnvVarEntry {
    fn to_crontab_entry(self) -> CrontabEntry {
        CrontabEntry::EnvVar(self)
    }
}

impl FromStr for EnvVarEntry {
    type Err = CrontabEntryParseError;
    fn from_str(s: &str) -> Result<EnvVarEntry, CrontabEntryParseError> {
        let spaces = [' ', '\t'];
        let mut splits = s.splitn(2, '=');

        let name = match splits.next() {
            Some(n) => n.trim_right_matches(&spaces[..]),
            None => return Err(CrontabEntryParseError)
        };

        if name.len() == 0 {
            return Err(CrontabEntryParseError)
        }

        if name.chars().any(|v| v == ' ' || v == '\t') {
            return Err(CrontabEntryParseError)
        }

        let mut value = match splits.next() {
            Some(v) => v.trim_left_matches(&spaces[..]),
            None => return Err(CrontabEntryParseError)
        };

        if value.len() > 1 {
            if &value[..1] == "'" || &value[..1] == "\"" && &value[..1] == &value[value.len()-1..] {
                value = &value[1..value.len()-1];
            }
        }

        Ok(EnvVarEntry(name.to_string(), value.to_string()))
    }
}

#[derive(Debug, PartialEq)]
// user, group, class
pub struct UserInfo(pub String, pub Option<String>, pub Option<String>);

pub struct UserInfoParseError;
impl FromStr for UserInfo {
    type Err = UserInfoParseError;
    fn from_str(s: &str) -> Result<UserInfo, UserInfoParseError> {
        let mut splits = s.split(':');
        Ok(UserInfo(
            try!(splits.next().ok_or(UserInfoParseError).map(ToString::to_string)),
            splits.next().map(ToString::to_string),
            splits.next().map(ToString::to_string)
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct CrontabEntryParseError;

impl FromStr for UserCrontabEntry {
    type Err = CrontabEntryParseError;

    fn from_str(s: &str) -> Result<UserCrontabEntry, CrontabEntryParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]).filter(|v| *v != "");
        Ok(UserCrontabEntry {
            sched: try!(<Result<Schedule, ScheduleParseError>>::from_iter(&mut splits).map_err(|_| CrontabEntryParseError)),
            cmd: splits.collect::<Vec<&str>>().connect(" ")
        })
    }
}

impl FromStr for RootCrontabEntry {
    type Err = CrontabEntryParseError;

    fn from_str(s: &str) -> Result<RootCrontabEntry, CrontabEntryParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]).filter(|v| *v != "");
        Ok(RootCrontabEntry {
            sched: try!(<Result<Schedule, ScheduleParseError>>::from_iter(&mut splits).map_err(|_| CrontabEntryParseError)),
            user: try!(splits.next().ok_or(UserInfoParseError).and_then(FromStr::from_str).map_err(|_| CrontabEntryParseError)),
            cmd: splits.collect::<Vec<&str>>().connect(" ")
        })
    }
}

impl FromStr for AnacrontabEntry {
    type Err = CrontabEntryParseError;

    fn from_str(s: &str) -> Result<AnacrontabEntry, CrontabEntryParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]).filter(|v| *v != "");
        Ok(AnacrontabEntry {
            period: try!(splits.next().ok_or(PeriodParseError).and_then(FromStr::from_str).map_err(|_| CrontabEntryParseError)),
            delay: try!(splits.next().and_then(|v| v.parse().ok()).map(Duration::minutes).ok_or(CrontabEntryParseError)),
            jobid: try!(splits.next().ok_or(CrontabEntryParseError).map(ToString::to_string)),
            cmd: splits.collect::<Vec<&str>>().connect(" ")
        })
    }
}
