extern crate rustc_serialize;
extern crate docopt;
extern crate users;
extern crate glob;

use docopt::Docopt;
use std::env;
use std::fs;
use std::io::{ErrorKind, stdin, stdout, Write, Read};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{PathBuf, Path};
use std::process::exit;

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
  -s, --show            Show all users who have a crontab.
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
    let mut stdout = stdout();
    loop {
        stdout.write_all(msg.as_bytes()).unwrap();
        stdout.flush();
        let mut buf = [0u8; 1024];
        match stdin.read(&mut buf) {
            Ok(n) if n > 0 && (buf[0] == 121 || buf[0] == 89) => return true,
            Ok(n) if n > 0 && (buf[0] == 110 || buf[0] == 78) => return false,
            _ => { stdout.write_all("Please reply \"y\" or \"n\"\n".as_bytes()).unwrap(); },
        }
    }
}

fn list(cron_file: &Path, args: &Args) {
    if let Err(e) = File::open(cron_file).map(|file| file.tee(stdout()).bytes().count()) {
        use std::io::ErrorKind::*;
        match e.kind() {
            NotFound => println!("no crontab for {}", args.flag_user),
            PermissionDenied => println!("you can not display {}'s crontab", args.flag_user),
            _ => println!("failed to read {}", cron_file),
        }
        exit(1);
    }
}

fn remove(cron_file: &Path, args: &Args) {
    if !args.flag_ask || confirm(&*format!("Are you sure you want to delete {} (y/n)? ", cron_file)) {
        if let Err(e) = fs::remove_file(cron_file) {
            use std::io::ErrorKind::*;
            match e.kind() {
                NotFound => println!("no crontab for {}", args.flag_user),
                PermissionDenied => match args.flag_user {
                    user @ Some(_) if user != users::get_current_username() => {
                        println!("you can not remove {}'s crontab", args.flag_user);
                    },
                    _ => match File::create(cron_file) {
                        Ok(_) => println!("couldn't remove {}, wiped it instead", cron_file),
                        Err(_) => println!("failed to remove {}", cron_file),
                    }
                },
                _ => println!("failed to remove {}", cron_file)
            }
            exit(1);
        }
    }
}

fn show(cron_file: &Path, args: &Args) {
    if users::get_current_uid() != 0 {
        return println!("must be privileged to use -s");
    }

    if let Ok(dir) = fs::read_dir(CRONTAB_DIR) {
        for entry in dir {
            let name = entry.ok().and_then(|e| e.path().file_name()).map(|s| s.to_string_lossy().into_owned());
            if let Some(user) = name {
                if users::get_user_by_name(&*user).is_some() {
                    println!("{}", user);
                } else {
                    println!("WARNING: crontab found with no matching user: {}", user)
                }
            }
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

    let cron_file = PathBuf::from(CRONTAB_DIR).join(args.flag_user.clone().or_else(|| users::get_current_username()).unwrap());

    if args.flag_show {
        show(&*cron_file, &args);
    } else if args.flag_list {
        list(&*cron_file, &args);
    }

    //println!("{:?}", confirm("Yes or no? "));
    //println!("{:?}", cron_file);
    //println!("{:?}", editor);
    //println!("{:?}", args);
}
