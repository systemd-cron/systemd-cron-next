use std::env::{current_exe};
use std::fs::{OpenOptions, File};
use std::io::{stderr, Stderr, Write};
use std::fmt::{Arguments, format};
use std::ffi::OsStr;
use std::borrow::Cow;
use libc::funcs::posix88::unistd;

fn current_name() -> String {
    current_exe().ok().and_then(|p| p.file_name().map(OsStr::to_string_lossy).map(Cow::into_owned)).unwrap_or("???".to_string())
}

#[repr(u8)]
#[derive(Debug, Copy, PartialOrd, PartialEq, Eq, Ord)]
pub enum LogLevel {
    Emergency = 0,
    Alert =  1,
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Info = 6 ,
    Debug = 7
}

pub trait Logger {
    fn new() -> Self;
    fn log(&mut self, level: LogLevel, msg: &str);
    fn log_fmt(&mut self, level: LogLevel, args: Arguments) {
        self.log(level, &*format(args));
    }
}

pub struct KernelLogger {
    name: String,
    kmsg: File
}
impl Logger for KernelLogger {
    fn new() -> KernelLogger {
        KernelLogger {
            name: current_name(),
            kmsg: OpenOptions::new().write(true).open("/dev/kmsg").unwrap()
        }
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        let data = format!("<{}>{}[{}]: {}\n", level as u8, self.name, unsafe { unistd::getpid() }, msg);
        self.kmsg.write_all(data.as_bytes()).unwrap();
    }
}

pub struct ConsoleLogger {
    name: String,
    stderr: Stderr
}

impl Logger for ConsoleLogger {
    fn new() -> ConsoleLogger {
        ConsoleLogger {
            name: current_name(),
            stderr: stderr()
        }
    }
    #[allow(unused_variables)]
    fn log(&mut self, level: LogLevel, msg: &str) {
        writeln!(&mut self.stderr, "{}: {}", self.name, msg).unwrap();
        let _ = self.stderr.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::{ConsoleLogger, KernelLogger, LogLevel, Logger};

    #[test]
    fn log_to_console() {
        let mut logger = ConsoleLogger::new();
        logger.log(LogLevel::Info, "test message");
    }

    #[test]
    fn log_to_kernel() {
        let mut logger = KernelLogger::new();
        logger.log(LogLevel::Info, "test message");
    }
}
