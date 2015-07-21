#![feature(convert)]
#![feature(io)]

extern crate rustc_serialize;
extern crate docopt;
extern crate users;
extern crate glob;
extern crate tempfile;
extern crate libc;

use tempfile::NamedTempFile;
use docopt::Docopt;
use std::env;
use std::fs;
use std::io::{stdin, stdout, stderr, Write, Read, self};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{PathBuf, Path};
use std::process::{Command, exit};
use std::ops::Deref;
use std::ffi::CString;

extern "C" {
    fn chown(path: *const libc::c_char, owner: libc::uid_t, group: libc::gid_t) -> libc::c_int;
}

fn change_owner<P: AsRef<Path>>(path: P, owner: libc::uid_t, group: libc::gid_t) -> Result<(), io::Error> {
    match unsafe { chown(CString::new(path.as_ref().as_os_str().to_bytes().unwrap()).unwrap().as_ptr(), owner, group) } {
        0 => Ok(()),
        -1 => Err(io::Error::last_os_error()),
        _ => unreachable!()
    }
}

static CRONTAB_DIR: &'static str = "/var/spool/cron";

static USAGE: &'static str = r#"
Usage: crontab [-u <user>] -l
       crontab [-u <user>] -e [<file>]
       crontab [-u <user>] -s
       crontab [-u <user>] -r [-i]
       crontab -h | --help

Maintain crontab files for individual users

Options:

  -h, --help                Show this help message and exit.
  -u <user>, --user <user>  It specifies the name of the user whose crontab is to
                            be tweaked. If this option is not given, crontab
                            examines "your" crontab, i.e., the crontab of the
                            person executing the command. Note that su(8) can
                            confuse crontab and that if you are running inside of
                            su(8) you should always use the -u option for safety's
                            sake. The first form of this command is used to
                            install a new crontab from some named file or standard
                            input if the pseudo-filename "-" is given.
  -l, --list                The current crontab will be displayed on standard
                            output.
  -r, --remove              The current crontab will be removed.
  -e, --edit                This option is used to edit the current crontab using
                            the editor specified by the VISUAL or EDITOR
                            environment variables. After you exit from the editor,
                            the modified crontab will be installed automatically.
  -s, --show                Show all users who have a crontab.
  -i, --ask                 This option modifies the -r option to prompt the user
                            for a 'y/Y' response before actually removing the
                            crontab.
"#;

#[derive(Debug, RustcDecodable)]
#[allow(non_snake_case)]
struct Args {
    arg_file: Option<String>,
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
            _ => println!("failed to read {}: {}", cron_file.display(), e),
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
                _ => writeln!(stderr, "failed to remove {}: {}", cron_file.display(), e)
            }.unwrap();
            return 1;
        }
    }
    0
}

fn show(_cron_file: &Path, _args: &Args) -> i32 {
    let mut stderr = stderr();

    if let Ok(dir) = fs::read_dir(CRONTAB_DIR) {
        for item in dir {
            if let Ok(entry) = item {
                if let Some(name) = entry.file_name().to_str() {
                    if users::get_user_by_name(name).is_some() {
                        println!("{}", name);
                    } else {
                        writeln!(stderr, "WARNING: crontab found with no matching user: {}", name).unwrap();
                    }
                }
            }
        }
    }
    0
}

fn edit(cron_file: &Path, _args: &Args) -> i32 {
    use std::io::ErrorKind::*;

    let mut stderr = stderr();

    let editor = match get_editor() {
        None => {
            writeln!(stderr, "no editor found").unwrap();
            return 1;
        },
        Some(editor) => editor
    };

    let mut tmpfile = NamedTempFile::new_in(CRONTAB_DIR).unwrap();

    if let Err(e) = File::open(cron_file).map(|file| file.tee(&mut tmpfile).bytes().count()) {
        match e.kind() {
            NotFound => tmpfile.write_all("# min hour dom month dow command".as_bytes()).unwrap(),
            _ => {
                writeln!(stderr, "error copying crontab file {}: {}", cron_file.display(), e).unwrap();
                return 1;
            }
        }
    }

    tmpfile.flush().unwrap();
    {
        let (uid, gid) = (users::get_current_uid(), users::get_current_gid());
        change_owner(tmpfile.path(), uid, gid).unwrap(); // TODO: user from args.flag_user
        let _guard = users::switch_user_group(uid, gid);
        match Command::new(editor).arg(tmpfile.path()).status() {
            Ok(status) if status.success() => (),
            _ => {
                writeln!(stderr, "edit aborted, your edit is kept here: {}", tmpfile.path().display()).unwrap();
                return 1;
            }
        }
    }

    // TODO: check tmpfile with parser

    if let Err(err) = tmpfile.persist(cron_file) {
        writeln!(stderr, "unexpected error: {}, your edit is kept here: {}", err.error, err.file.path().display()).unwrap();
        return 1;
    }

    0
}

fn replace(cron_file: &Path, args: &Args) -> i32 {
    let mut tmpfile = NamedTempFile::new_in(CRONTAB_DIR).unwrap();

    match args.arg_file {
        Some(ref name) if &**name == "-" => { stdin().tee(&mut tmpfile).bytes().count(); },
        Some(ref name) => { File::open(&**name).unwrap().tee(&mut tmpfile).bytes().count(); },
        None => unreachable!()
    }

    tmpfile.flush().unwrap();

    // TODO: check tmpfile syntax

    if let Err(e) = tmpfile.persist(cron_file) {
        writeln!(stderr(), "error renaming {} to {}: {}", e.file.path().display(), cron_file.display(), e.error).unwrap();
        return 1;
    }

    // TODO: user from args.flag_user
    change_owner(cron_file, users::get_current_uid(), users::get_current_gid()).unwrap();

    0
}

fn main() {
    let mut stderr = stderr();
    let mut args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    if args.flag_user.is_some() && users::get_current_uid() != 0 {
        writeln!(stderr, "must be privileged to use -u").unwrap();
        exit(1);
    }

    match fs::metadata(CRONTAB_DIR) {
        Ok(ref meta) if !meta.is_dir() => {
            writeln!(stderr, "{} is not a directory!", CRONTAB_DIR).unwrap();
            exit(1);
        },
        Err(_) => if let Err(_) = fs::create_dir_all(CRONTAB_DIR) {
            writeln!(stderr, "{} doesn't exist!", CRONTAB_DIR).unwrap();
            exit(1);
        },
        _ => ()
    }

    if args.flag_user.is_none() {
        args.flag_user = users::get_current_username();
    }
    let cron_file = PathBuf::from(CRONTAB_DIR).join(args.flag_user.clone().unwrap());

    exit(match args {
        Args { flag_show: true, .. } => show(&*cron_file, &args),
        Args { flag_list: true, .. } => list(&*cron_file, &args),
        Args { flag_edit: true, arg_file: None, .. } => edit(&*cron_file, &args),
        Args { flag_edit: true, .. } => replace(&*cron_file, &args),
        Args { flag_remove: true, .. } => remove(&*cron_file, &args),
        _ => unreachable!()
    })
}
