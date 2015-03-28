use std::time::Duration;
use std::str::FromStr;

use schedule::{Schedule, Period, Calendar};
use interval::IntervalParseError;

#[derive(Debug)]
pub enum CrontabEntry {
    User(UserCrontabEntry),
    Root(RootCrontabEntry),
    Anacron(AnacrontabEntry)
}

#[derive(Debug)]
pub struct UserCrontabEntry {
    pub sched: Schedule,
    pub cmd: String
}

#[derive(Debug)]
pub struct RootCrontabEntry {
    pub sched: Schedule,
    pub user: UserInfo,
    pub cmd: String
}

#[derive(Debug)]
pub struct AnacrontabEntry {
    pub period: Period,
    pub delay: Duration,
    pub jobid: String,
    pub cmd: String
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct CrontabEntryParseError;

macro_rules! parse_cron_rec_field {
    ($iter:expr, $err:ident) => {
        try!($iter.next().ok_or($err).and_then(FromStr::from_str).map_err(|_| CrontabEntryParseError))
    };
    ($iter:expr) => {
        parse_cron_rec_field!($iter, IntervalParseError)
    };
}

impl FromStr for UserCrontabEntry {
    type Err = CrontabEntryParseError;

    fn from_str(s: &str) -> Result<UserCrontabEntry, CrontabEntryParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]);
        Ok(UserCrontabEntry {
            sched: Schedule::Calendar(Calendar {
                mins: parse_cron_rec_field!(splits),
                hrs: parse_cron_rec_field!(splits),
                days: parse_cron_rec_field!(splits),
                mons: parse_cron_rec_field!(splits),
                dows: parse_cron_rec_field!(splits),
            }),
            cmd: splits.collect::<Vec<&str>>().connect(" ")
        })
    }
}

impl FromStr for RootCrontabEntry {
    type Err = CrontabEntryParseError;

    fn from_str(s: &str) -> Result<RootCrontabEntry, CrontabEntryParseError> {
        let seps = [' ', '\t'];
        let mut splits = s.split(&seps[..]);
        Ok(RootCrontabEntry {
            sched: Schedule::Calendar(Calendar {
                mins: parse_cron_rec_field!(splits),
                hrs: parse_cron_rec_field!(splits),
                days: parse_cron_rec_field!(splits),
                mons: parse_cron_rec_field!(splits),
                dows: parse_cron_rec_field!(splits),
            }),
            user: parse_cron_rec_field!(splits, UserInfoParseError),
            cmd: splits.collect::<Vec<&str>>().connect(" ")
        })
    }
}
