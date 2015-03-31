use std::convert::AsRef;
use std::fs::{walk_dir, PathExt};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use cronparse::{CrontabFile, CrontabFileError, CrontabFileErrorKind};
use cronparse::crontab::{EnvVarEntry, CrontabEntry, ToCrontabEntry};
use log::{Logger, LogLevel};

pub fn process_crontab_dir<T: ToCrontabEntry>(dir: &str, logger: &mut Logger) {
    let files = walk_dir(dir).and_then(|fs| fs.map(|r| r.map(|p| p.path()))
                                       .filter(|r| r.as_ref().map(|p| p.is_file()).unwrap_or(true))
                                       .collect::<Result<Vec<PathBuf>, _>>());
    match files {
        Err(err) => log!(logger, LogLevel::Error, "error processing directory {}: {}", dir, err),
        Ok(files) => for file in files {
            process_crontab_file::<T, _>(file, logger as &mut Logger);
        }
    }
}


pub fn process_crontab_file<T: ToCrontabEntry, P: AsRef<Path>>(path: P, logger: &mut Logger) {
    CrontabFile::<T>::new(path.as_ref()).map(|crontab| {
        let mut env = BTreeMap::new();
        for entry in crontab {
            match entry {
                Ok(CrontabEntry::EnvVar(EnvVarEntry(name, value))) => { env.insert(name, value); },
                Ok(data) => generate_systemd_units(path.as_ref(), data, &env, logger),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Io(_), .. }) => log!(logger, LogLevel::Warning, "error accessing file {}: {}", path.as_ref().display(), err),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Parse(_), .. }) => log!(logger, LogLevel::Warning, "skipping file {} due to parsing error: {}", path.as_ref().display(), err),
            }
        }
    }).unwrap_or_else(|err| {
        log!(logger, LogLevel::Error, "error parsing file {}: {}", path.as_ref().display(), err);
    });
}

fn generate_systemd_units(path: &Path, entry: CrontabEntry, env: &BTreeMap<String, String>, logger: &mut Logger) {
    log!(logger, LogLevel::Info, "{} => {:?}, {:?}", path.display(), entry, env);
}

