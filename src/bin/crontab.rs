#![feature(io)]

extern crate rustc_serialize;
extern crate docopt;
extern crate users;
extern crate glob;
extern crate tempfile;

use tempfile::NamedTempFile;
use docopt::Docopt;
use std::env;
use std::fs;
use std::io::{stdin, stdout, stderr, Write, Read};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{PathBuf, Path};
use std::process::{Command, exit};
use std::ops::Deref;

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
        stdout.flush().unwrap();
        let mut buf = [0u8; 1024];
        match stdin.read(&mut buf) {
            Ok(n) if n > 0 && (buf[0] == 121 || buf[0] == 89) => return true,
            Ok(n) if n > 0 && (buf[0] == 110 || buf[0] == 78) => return false,
            _ => { stdout.write_all("Please reply \"y\" or \"n\"\n".as_bytes()).unwrap(); },
        }
    }
}

fn list(cron_file: &Path, args: &Args) -> i32 {
    if let Err(e) = File::open(cron_file).map(|file| file.tee(stdout()).bytes().count()) {
        use std::io::ErrorKind::*;
        match e.kind() {
            NotFound => println!("no crontab for {}", args.flag_user.as_ref().map(String::deref).unwrap_or("???")),
            PermissionDenied => println!("you can not display {}'s crontab", args.flag_user.as_ref().map(String::deref).unwrap_or("???")),
            _ => println!("failed to read {}", cron_file.display()),
        }
        return 1;
    }
    0
}

fn remove(cron_file: &Path, args: &Args) -> i32 {
    let mut stderr = stderr();

    if !args.flag_ask || confirm(&*format!("Are you sure you want to delete {} (y/n)? ", cron_file.display())) {
        if let Err(e) = fs::remove_file(cron_file) {
            use std::io::ErrorKind::*;
            match e.kind() {
                NotFound => writeln!(stderr, "no crontab for {}", args.flag_user.as_ref().map(String::deref).unwrap_or("???")),
                PermissionDenied => match args.flag_user {
                    ref user @ Some(_) if user != &users::get_current_username() => writeln!(stderr, "you can not remove {}'s crontab", args.flag_user.as_ref().map(String::deref).unwrap_or("???")),
                    _ => match File::create(cron_file) {
                        Ok(_) => writeln!(stderr, "couldn't remove {}, wiped it instead", cron_file.display()),
                        Err(_) => writeln!(stderr, "failed to remove {}", cron_file.display()),
                    }
                },
                _ => writeln!(stderr, "failed to remove {}", cron_file.display())
            }.unwrap();
            return 1;
        }
    }
    0
}

fn show(cron_file: &Path, args: &Args) -> i32 {
    let mut stderr = stderr();

    if users::get_current_uid() != 0 {
        writeln!(stderr, "must be privileged to use -s");
        return 2;
    }

    if let Ok(dir) = fs::read_dir(CRONTAB_DIR) {
        for entry in dir {
            let name = entry.ok().and_then(|e| e.path().file_name().map(|s| s.to_string_lossy().into_owned()));
            if let Some(user) = name {
                if users::get_user_by_name(&*user).is_some() {
                    println!("{}", user);
                } else {
                    writeln!(stderr, "WARNING: crontab found with no matching user: {}", user);
                }
            }
        }
    }
    0
}

fn edit(cron_file: &Path, args: &Args) -> i32 {
    use std::io::ErrorKind::*;

    let mut stderr = stderr();

    let editor = match get_editor() {
        None => {
            writeln!(stderr, "no editor found");
            return 1;
        },
        Some(editor) => editor
    };

    let mut tmpfile = NamedTempFile::new_in(CRONTAB_DIR).unwrap();

    if let Err(e) = File::open(cron_file).map(|file| file.tee(&mut tmpfile).bytes().count()) {
        match e.kind() {
            NotFound => tmpfile.write_all("# min hour dom month dow command".as_bytes()).unwrap(),
            _ => {
                writeln!(stderr, "you can not edit {}'s crontab", args.flag_user.as_ref().map(String::deref).unwrap_or("???"));
                return 1;
            }
        }
    }

    tmpfile.flush().unwrap();
    match Command::new(editor).arg(tmpfile.path()).status() {
        Ok(status) if status.success() => (),
        _ => {
            writeln!(stderr, "edit aborted, your edit is kept here: {}", tmpfile.path().display());
            return 1;
        }
    }

    // TODO: check tmpfile with parser

    if let Err(err) = tmpfile.persist(cron_file) {
        match err.error.kind() {
            PermissionDenied => writeln!(stderr, "you can not edit {}'s crontab, your edit is kept here: {}",
                                         args.flag_user.as_ref().map(String::deref).unwrap_or("???"),
                                         err.file.path().display()),
            _ => writeln!(stderr, "unexpected error: {}, your edit is kept here: {}", err.error, err.file.path().display())
        }.unwrap();
        return 1;
    }

    0
}

fn main() {
    let mut args: Args = Docopt::new(USAGE)
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

    if args.flag_user.is_none() {
        args.flag_user = users::get_current_username();
    }
    let cron_file = PathBuf::from(CRONTAB_DIR).join(args.flag_user.clone().unwrap());

    exit(
        if args.flag_show {
            show(&*cron_file, &args)
        } else if args.flag_list {
            list(&*cron_file, &args)
        } else if args.flag_edit {
            edit(&*cron_file, &args)
        } else if args.flag_remove {
            remove(&*cron_file, &args)
        } else {
            0
        })
}
