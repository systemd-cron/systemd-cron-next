#![feature(phase)]

extern crate core;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

use std::os;
use std::io::{BufferedStream, File, Lines};
use std::io::fs::readdir;
use std::collections::HashMap;

use regex::Regex;
use core::iter::Enumerate;

static CRONTAB_DIR: &'static str = "/etc/crontab";
static ANACRONTAB_DIR: &'static str = "/etc/anacrontab";
static USERCRONTAB_DIR: &'static str = "/var/spool/cron";
static ETCCRONTAB_DIR: &'static str = "/etc/cron.d";

static SPACES: Regex = regex!("[ \t]+");

enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

type Minute = u8;
type Day = u8;
type Hour = u8;
type Delay = u8;

enum Period {
    Reboot,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

// 'e - lifetime for CrontabEntry
// 'i - lifetime for CrontabIterator

struct CrontabEntry<'e> {
    environment: &'e HashMap<&'e str, &'e str>,
    lineno: uint,
    filename: &'e Path,
    username: &'e str,
    command: &'e str,

    period: Option<Period>,
    delay: Option<Delay>,

    minutes: Option<Vec<Minute>>,
    hours: Option<Vec<Hour>>,
    days: Option<Vec<Day>>,
    dows: Option<Vec<DayOfWeek>>,
    months: Option<Vec<Month>>,
}

struct CrontabIterator<'i> {
    filename: &'i Path,
    lines: Enumerate<Lines<'i, BufferedStream<File>>>,
    environment: HashMap<&'i str, &'i str>,
}

impl<'i> CrontabIterator<'i> {
    fn new(filename: &'i Path) -> CrontabIterator<'i> {
        CrontabIterator {
            filename: filename,
            lines: BufferedStream::new(File::open(filename).unwrap()).lines().enumerate(),
            environment: HashMap::new(),
        }
    }
}

impl<'i> Iterator<CrontabEntry<'i>> for CrontabIterator<'i> {
    fn next<'i>(&mut self) -> Option<CrontabEntry<'i>> {
        match self.lines.next() {
            None => None,
            Some((_, Err(_))) => {
                None
            },

            Some((lineno, Ok(line))) => {
                let parts = SPACES.split(line.as_slice()).collect::<Vec<&str>>();
                Some(CrontabEntry {
                    environment: &self.environment,
                    lineno: lineno,
                    filename: self.filename,
                    username: "root",
                    command: "",

                    period: None,
                    delay: None,

                    minutes: None,
                    hours: None,
                    days: None,
                    dows: None,
                    months: None,
                })
            }
        }
    }
}

fn main() {
    let args: Vec<String> = os::args();

    println!("target dir: {}", args.get(1));
}
