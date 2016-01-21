use std::convert::AsRef;
use std::fs::{read_dir, metadata};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::str::FromStr;

use cronparse::{CrontabFile, CrontabFileError, CrontabFileErrorKind};
use cronparse::crontab::{EnvVarEntry, CrontabEntry};

use generate::generate_systemd_units;

pub fn process_crontab_dir<T: FromStr, D: AsRef<Path>>(srcdir: &str, dstdir: D) where CrontabEntry: From<T>, CrontabFileError: From<<T as FromStr>::Err> {
    let files = read_dir(srcdir).and_then(|fs| fs.map(|r| r.map(|p| p.path()))
                                       .filter(|r| r.as_ref().map(|p| metadata(p).map(|m| m.is_file()).unwrap_or(true)).unwrap_or(true))
                                       .collect::<Result<Vec<PathBuf>, _>>());
    match files {
        Err(err) => warn!("error processing directory {}: {}", srcdir, err),
        Ok(files) => for file in files {
            process_crontab_file::<T, _, _>(file, dstdir.as_ref());
        }
    }
}

pub fn process_crontab_file<T: FromStr, P: AsRef<Path>, D: AsRef<Path>>(path: P, dstdir: D) where CrontabEntry: From<T>, CrontabFileError: From<<T as FromStr>::Err> {
    CrontabFile::<T>::new(path.as_ref()).map(|crontab| {
        let mut env = BTreeMap::new();
        for entry in crontab {
            match entry {
                Ok(CrontabEntry::EnvVar(EnvVarEntry(name, value))) => { env.insert(name, value); },
                Ok(data) => match generate_systemd_units(data, &env, path.as_ref(), dstdir.as_ref()) {
                    Ok(_) => (),
                    Err(err) => warn!("error generating unit from {}: {}", path.as_ref().display(), err)
                },
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Io(_), .. }) => warn!("error accessing file {}: {}", path.as_ref().display(), err),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Parse(_), .. }) => warn!("skipping file {} due to parsing error: {}", path.as_ref().display(), err),
            }
        }
    }).unwrap_or_else(|err| {
        warn!("error parsing file {}: {}", path.as_ref().display(), err);
    });
}

