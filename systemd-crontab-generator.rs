use std::os;
use std::io::{BufferedStream, File, IoResult};

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

impl<'i> Iterator<String> for CrontabIterator<'i> {
    fn next(&mut self) -> Option<String> {
        match self.lines.next() {
            None => None,
            Some(Err(_)) => None,
            Some(Ok(line)) => Some(line),
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
