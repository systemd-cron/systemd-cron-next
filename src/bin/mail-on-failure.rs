#[macro_use]
extern crate log;
extern crate kernlog;

use std::env;
use std::process::{Command, Stdio};
use std::io::Write;

macro_rules! try_log {
    ($exp:expr) => {
        match $exp {
            Ok(v) => v,
            Err(e) => { warn!("{}", e); return; }
        }
    }
}

fn get_systemd_unit_property(unit: &str, prop: &str) -> String {
    String::from_utf8_lossy(&try_log!(Command::new("systemctl")
                            .arg("show")
                            .arg(unit)
                            .arg("--property")
                            .arg(prop)
                            .output())
                            .stdout[prop.len() + 1..])
        .trim_right_matches('\n')
        .to_owned()
}

fn main() {
    kernlog::init().unwrap();

    let unit = match env::args().nth(1) {
        Some(unit) => unit,
        None => {
            println!("Usage: mail-on-failure <unit>");
            return;
        }
    };

    let mut user = get_systemd_unit_property(&*unit, "User");
    if user.len() == 0 {
        user = "root".to_owned();
    }


    let job_env = get_systemd_unit_property(&*unit, "Environment");
    for pair in job_env.split(' ') {
        let mut p = pair.splitn(2, '=');
        if let (Some(name), Some(value)) = (p.next(), p.next()) {
            if name == "MAILTO" {
                user = value.to_owned();
                break;
            }
        }
    }

    if user.len() == 0 {
        return;
    }

    let mut hostname = String::from_utf8_lossy(&try_log!(Command::new("uname")
        .arg("-n")
        .output())
        .stdout[..])
        .trim_right_matches('\n')
        .to_owned();

    if hostname.len() == 0 {
        hostname = "localhost".to_owned();
    }

    let mut head = String::new();
    head.push_str("From: root (systemd-cron)\nTo: ");
    head.push_str(&*user);
    head.push_str("\nSubject: [");
    head.push_str(&*hostname);
    head.push_str("] job ");
    head.push_str(&*unit);
    head.push_str(r###" failed
MIME-Version: 1.0
Content-Type: text/plain; charset=UTF-8
Content-Transfer-Encoding: 8bit
Auto-Submitted: auto-generated

"###);

    let status = Command::new("systemctl")
        .arg("status")
        .arg(&*unit)
        .output()
        .unwrap();

    let mut mailer = try_log!(Command::new("sendmail")
        .arg("-i")
        .arg("-B8BITMIME")
        .arg(&*user)
        .stdin(Stdio::piped())
        .spawn());

    if let Some(ref mut stdin) = mailer.stdin {
        try_log!(stdin.write_all(head.as_bytes()).and_then(|_|
                 stdin.write_all(&*status.stdout)));
    }

    mailer.wait().unwrap();
}
