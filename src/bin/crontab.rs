extern crate rustc_serialize;
extern crate docopt;
extern crate users;

use docopt::Docopt;
use users::{Users, OSUsers};
use std::env;
use std::fs;
use std::io::{stdin, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

static CRONTAB_DIR: &'static str = "/var/spool/cron";
static REBOOT_FILE: &'static str = "/run/crond.reboot";

static USAGE: &'static str = r#"
Usage: crontab [-u USER] -l
       crontab [-u USER] -e [FILE]
       crontab [-u USER] -s
       crontab [-u USER] -r [-i]
       crontab -h | --help

Maintain crontab files for individual users

Options:

  -h, --help            Show this help message and exit.
  -u USER, --user USER  It specifies the name of the user whose crontab is to
                        be tweaked. If this option is not given, crontab
                        examines "your" crontab, i.e., the crontab of the
                        person executing the command. Note that su(8) can
                        confuse crontab and that if you are running inside of
                        su(8) you should always use the -u option for safety's
                        sake. The first form of this command is used to
                        install a new crontab from some named file or standard
                        input if the pseudo-filename "-" is given.
  -l, --list            The current crontab will be displayed on standard
                        output.
  -r, --remove          The current crontab will be removed.
  -e, --edit            This option is used to edit the current crontab using
                        the editor specified by the VISUAL or EDITOR
                        environment variables. After you exit from the editor,
                        the modified crontab will be installed automatically.
  -s, --show            Show all user who have a crontab.
  -i, --ask             This option modifies the -r option to prompt the user
                        for a 'y/Y' response before actually removing the
                        crontab.
"#;

#[derive(Debug, RustcDecodable)]
#[allow(non_snake_case)]
struct Args {
    arg_FILE: Option<String>,
    flag_user: Option<String>,
    flag_list: bool,
    flag_remove: bool,
    flag_edit: bool,
    flag_show: bool,
    flag_ask: bool
}

fn get_editor() -> Option<String> {
    env::var("EDITOR").ok()
        .or_else(|| env::var("VISUAL").ok())
        .or_else(|| ["/usr/bin/editor", "/usr/bin/vim", "/usr/bin/nano", "/usr/bin/mcedit"].iter()
                 .filter(|editor| fs::metadata(editor).map(|meta| meta.is_file() && meta.permissions().mode() & 0o0111 != 0).unwrap_or(false))
                 .next()
                 .map(|&s| s.to_owned()))
}

fn confirm(msg: &str) -> bool {
    let mut stdin = stdin();
    loop {
        print!("{}", msg);
        let mut buf = [0u8; 1];
        match stdin.read(&mut buf) {
            Ok(1) => return buf[0] == 121 || buf[0] == 89,
            _ => println!("Please reply \"y\" or \"n\""),
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let editor = get_editor();

    match fs::metadata(CRONTAB_DIR) {
        Ok(ref meta) if meta.is_dir() => (),
        Ok(_) => return println!("{} is not a directory!", CRONTAB_DIR),
        Err(_) => if let Err(_) = fs::create_dir_all(CRONTAB_DIR) {
            return println!("{} doesn't exist!", CRONTAB_DIR);
        }
    }

    let mut users = OSUsers::empty_cache();
    let cron_file = PathBuf::from(CRONTAB_DIR).join(args.flag_user.clone().or_else(|| users.get_current_username()).unwrap());

    println!("{:?}", cron_file);
    println!("{:?}", editor);
    println!("{:?}", args);
}
