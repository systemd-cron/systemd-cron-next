use std::time::Duration;
use std::str::FromStr;
use std::iter::FromIterator;

use schedule::{Schedule, Period, ScheduleParseError};

#[derive(Debug, PartialEq)]
pub enum CrontabEntry {
    User(UserCrontabEntry),
    Root(RootCrontabEntry),
    Anacron(AnacrontabEntry)
}

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
