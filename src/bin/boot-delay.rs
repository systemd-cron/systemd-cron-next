#![feature(slice_position_elem)]
#![feature(duration)]
#![feature(thread_sleep)]

use std::fs::File;
use std::io::Read;
use std::mem::transmute;
use std::thread::sleep;
use std::time::Duration;
use std::env;

fn main() {
    let delay = match env::args().nth(1).and_then(|s| s.parse::<f32>().ok()) {
        Some(d) => 60.0 * d,
        None => {
            println!("Usage: boot-delay <minutes>");
            return;
        }
    };

    let mut buf = [0u8; 1024];
    let uptime = File::open("/proc/uptime")
        .and_then(|ref mut file| file.read(&mut buf))
        .map(|sz| {
            buf.position_elem(&0x20)
               .and_then(|p| if p < sz {
                   unsafe { transmute::<_, &str>(&buf[..p]) }.parse::<f32>().ok()
               } else {
                   None
               }).unwrap()
        })
        .unwrap();

    if delay > uptime {
        sleep(Duration::from_secs((delay - uptime) as u64));
    }
}
