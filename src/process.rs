use std::convert::AsRef;
use std::fs::{read_dir, PathExt, metadata};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use cronparse::{CrontabFile, CrontabFileError, CrontabFileErrorKind};
use cronparse::crontab::{EnvVarEntry, CrontabEntry, ToCrontabEntry};

use generate::generate_systemd_units;

pub fn process_crontab_dir<T: ToCrontabEntry, D: AsRef<Path>>(srcdir: &str, dstdir: D) {
    let files = read_dir(srcdir).and_then(|fs| fs.map(|r| r.map(|p| p.path()))
                                       .filter(|r| r.as_ref().map(|p| metadata(p).map(|m| m.is_file()).unwrap_or(true)).unwrap_or(true))
                                       .collect::<Result<Vec<PathBuf>, _>>());
    match files {
        Err(err) => error!("error processing directory {}: {}", srcdir, err),
        Ok(files) => for file in files {
            process_crontab_file::<T, _, _>(file, dstdir.as_ref());
        },
    }
}

pub fn process_crontab_file<T: ToCrontabEntry, P: AsRef<Path>, D: AsRef<Path>>(path: P, dstdir: D) {
    CrontabFile::<T>::new(path.as_ref()).map(|crontab| {
        let mut env = BTreeMap::new();
        for entry in crontab {
            match entry {
                Ok(CrontabEntry::EnvVar(EnvVarEntry(name, value))) => { env.insert(name, value); },
                Ok(data) => match generate_systemd_units(data, &env, path.as_ref(), dstdir.as_ref()) {
                    Ok(_) => (),
                    Err(err) => error!("error generating unit from {}: {}", path.as_ref().display(), err)
                },
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Io(_), .. }) => warn!("error accessing file {}: {}", path.as_ref().display(), err),
                Err(err @ CrontabFileError { kind: CrontabFileErrorKind::Parse(_), .. }) => warn!("skipping file {} due to parsing error: {}", path.as_ref().display(), err),
            }
        }
    }).unwrap_or_else(|err| {
        error!("error parsing file {}: {}", path.as_ref().display(), err);
    });
}
