extern crate cronparse;

use cronparse::crontab::{SystemCrontabEntry, UserCrontabEntry, UserInfo, EnvVarEntry, CrontabEntryParseError};
use cronparse::schedule::{Calendar, Schedule, Minute, Hour, Day, Period};
use cronparse::schedule::DayOfWeek::*;
use cronparse::schedule::Month::*;
use cronparse::interval::Interval;

#[test]
fn parse_root_crontab_line_calendar_stars() {
    assert_eq!("* *\t*  * \t* root command with args".parse::<SystemCrontabEntry>(), Ok(SystemCrontabEntry {
        sched: Schedule::Calendar(Calendar {
            mins: vec![Interval::Full(1)],
            hrs: vec![Interval::Full(1)],
            days: vec![Interval::Full(1)],
            mons: vec![Interval::Full(1)],
            dows: vec![Interval::Full(1)],
        }),
        user: UserInfo("root".to_string(), None, None),
        cmd: "command with args".to_string()
    }));
}

#[test]
fn parse_root_crontab_line_calendar() {
    assert_eq!("*/15,1 1-5/2\t1,2,*/5  Jan-3/3,Feb \tSun,1,4-Fri/2 user command with args".parse::<SystemCrontabEntry>(), Ok(SystemCrontabEntry {
        sched: Schedule::Calendar(Calendar {
            mins: vec![Interval::Full(15), Interval::Value(Minute(1))],
            hrs: vec![Interval::Range(Hour(1), Hour(5), 2)],
            days: vec![Interval::Value(Day(1)), Interval::Value(Day(2)), Interval::Full(5)],
            mons: vec![Interval::Range(January, March, 3), Interval::Value(February)],
            dows: vec![Interval::Value(Sunday), Interval::Value(Monday), Interval::Range(Thursday, Friday, 2)],
        }),
        user: UserInfo("user".to_string(), None, None),
        cmd: "command with args".to_string()
    }));
}

#[test]
fn parse_user_crontab_line_calendar() {
    assert_eq!("*/15,1 1-5/2\t1,2,*/5  Jan-3/3,Feb \tSun,1,4-Fri/2 user command with args".parse::<UserCrontabEntry>(), Ok(UserCrontabEntry {
        sched: Schedule::Calendar(Calendar {
            mins: vec![Interval::Full(15), Interval::Value(Minute(1))],
            hrs: vec![Interval::Range(Hour(1), Hour(5), 2)],
            days: vec![Interval::Value(Day(1)), Interval::Value(Day(2)), Interval::Full(5)],
            mons: vec![Interval::Range(January, March, 3), Interval::Value(February)],
            dows: vec![Interval::Value(Sunday), Interval::Value(Monday), Interval::Range(Thursday, Friday, 2)],
        }),
        cmd: "user command with args".to_string()
    }));
}

#[test]
fn parse_user_crontab_line_period() {
    assert_eq!("@reboot \t user command with args".parse::<UserCrontabEntry>(), Ok(UserCrontabEntry {
        sched: Schedule::Period(Period::Reboot),
        cmd: "user command with args".to_string()
    }));
}

#[test]
fn parse_envvar_line() {
    assert_eq!("BATCH  =\t1".parse::<EnvVarEntry>(), Ok(EnvVarEntry("BATCH".to_string(), "1".to_string())));
    assert_eq!("BATCH  =\"1\"".parse::<EnvVarEntry>(), Ok(EnvVarEntry("BATCH".to_string(), "1".to_string())));
    assert_eq!("* * * * * test".parse::<EnvVarEntry>(), Err(CrontabEntryParseError::InvalidEnvVarName));
}
