#![feature(phase)]

#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::os;
use std::io::{BufferedStream, File, IoResult};
use std::from_str::FromStr;
use regex::Regex;
use std::num::from_int;

static SPACES: Regex = regex!("[ \t]+");

#[deriving(Show)]
enum Month {
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
    December = 12,
}

#[deriving(Show, FromPrimitive)]
enum DayOfWeek {
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}

#[deriving(Show)]
enum Period {
    Reboot,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl FromStr for Period {
    fn from_str(s: &str) -> Option<Period> {
        match s {
            "@reboot"  => Some(Reboot),
            "@daily"   => Some(Daily),
            "@weekly"  => Some(Weekly),
            "@monthly" => Some(Monthly),
            "@yearly"  => Some(Yearly),
            "@midnight" => Some(Daily),
            "@annually" => Some(Yearly),
            "1" => Some(Daily),
            "7" => Some(Weekly),
            _ => None,
        }
    }
}

impl FromStr for DayOfWeek {
    fn from_str(s: &str) -> Option<DayOfWeek> {
        match s.slice(0, 3).chars().map(|c| c.to_lowercase()).collect::<String>().as_slice() {
            "sun" => Some(Sunday),
            "mon" => Some(Monday),
            "tue" => Some(Tuesday),
            "wed" => Some(Wednesday),
            "thu" => Some(Thursday),
            "fri" => Some(Friday),
            "sat" => Some(Saturday),
            v => match from_str::<int>(v) {
                Some(n) => from_int::<DayOfWeek>(n % 7),
                None => None,
            },
        }
    }
}

#[deriving(Show)]
struct Minute(uint);

#[deriving(Show)]
struct Hour(uint);

#[deriving(Show)]
struct Day(uint);

// name, group, class
#[deriving(Show)]
struct User(String, Option<String>, Option<String>);

#[deriving(Show)]
struct Command(String);

#[deriving(Show)]
struct Delay(uint);

#[deriving(Show)]
struct JobId(String);

#[deriving(Show)]
enum CrontabEntry {
    Calendar(Vec<Minute>, Vec<Hour>, Vec<Day>, Vec<Month>, Vec<DayOfWeek>, Option<User>, Command),
    Monotonic(Period, Option<User>, Command),
    Anacron(Period, Delay, JobId, Command),
}

trait RangeEntry<T> {
    fn everything(&self) -> Vec<T>;

    fn to_range(&self, s: &str) -> Vec<T> {
        let all = self.everything();
        if s == "*" {
            return all;
        }

        // TODO: split `s` by comma, detect and exapnd ranges, map to all
    }
}

impl CrontabEntry {
    fn new(parts: Vec<String>) -> Option<CrontabEntry> {
        match parts.len() {
            6 => {
                Some(Calendar(
                    vec![Minute(0)],
                    vec![Hour(0)],
                    vec![Day(0)],
                    vec![January],
                    vec![Sunday],
                    Some(User("root".into_string(), None, None)),
                    Command("command".into_string()),
                ))
            },
            7 => {
                Some(Calendar(
                    vec![Minute(0)],
                    vec![Hour(0)],
                    vec![Day(0)],
                    vec![January],
                    vec![Sunday],
                    Some(User("root".into_string(), None, None)),
                    Command("command".into_string()),
                ))
            }
            _ => None
        }
    }
}

struct CrontabIterator<'i> {
    lines: &'i mut Iterator<IoResult<String>>,
}

impl<'i> CrontabIterator<'i> {
    fn new(iter: &'i mut Iterator<IoResult<String>>) -> CrontabIterator<'i> {
        CrontabIterator {
            lines: iter
        }
    }
}

impl<'i> Iterator<CrontabEntry> for CrontabIterator<'i> {
    fn next(&mut self) -> Option<CrontabEntry> {
        loop {
            match self.lines.next() {
                None => return None,
                Some(Err(_)) => return None,

                Some(Ok(line)) => {
                    let line = line.as_slice().trim();

                    if line.starts_with("#") {
                        continue;
                    }

                    let parts = SPACES.splitn(line, 6).map(|p| p.into_string()).collect();

                    return Some(CrontabEntry::new(parts).unwrap());
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = os::args();

    let file = File::open(&Path::new("/var/spool/cron/kstep")).unwrap();
    let mut buffer = BufferedStream::new(file);
    let mut lines = buffer.lines();
    let mut crontab = CrontabIterator::new(&mut lines);

    println!("target dir: {}", args.get(1));

    for line in crontab {
        println!("line: {}", line);
    }
}
