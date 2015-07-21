extern crate rumblebars;
extern crate rustc_serialize;

use std::env;
use std::path::Path;
use std::fs::{self, File};
use std::io::Read;
use std::collections::BTreeMap;

use rumblebars::{Template, EvalContext};
use rustc_serialize::json::Json;

static UNITS_DIR: &'static str = "units";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let output = Path::new(&*out_dir);

    let data = Json::Object(build_render_data());
    let ctx = EvalContext::new();

    for entry in fs::read_dir(UNITS_DIR).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        let target = output.join(&name[..name.len()-3]);
        let tmpl = File::open(entry.path()).and_then(|mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf).map(|_| Template::new(&*buf).unwrap())
        }).unwrap();

        println!("generating unit: {:?} -> {:?}...", entry.path(), target);
        tmpl.eval(&data, &mut File::create(target).unwrap(), &ctx).unwrap();
    }
}

fn build_render_data() -> BTreeMap<String, Json> {
    let mut ctx = BTreeMap::new();

    let prefix = env::var("PREFIX").unwrap_or_else(|_| "/usr/local".to_owned());
    let package = "systemd-cron";

    ctx.insert("package".to_owned(), Json::String(package.to_owned()));

    ctx.insert("bindir".to_owned(), Json::String(env::var("BIN_DIR").unwrap_or_else(|_| prefix.clone() + "/bin")));
    ctx.insert("confdir".to_owned(), Json::String(env::var("CONF_DIR").unwrap_or_else(|_| prefix.clone() + "/etc")));

    let datadir = env::var("DATA_DIR").unwrap_or_else(|_| prefix.clone() + "/share");
    let libdir = env::var("LIB_DIR").unwrap_or_else(|_| prefix.clone() + "/lib");

    ctx.insert("mandir".to_owned(), Json::String(env::var("MAN_DIR").unwrap_or_else(|_| datadir.clone() + "/man")));
    ctx.insert("docdir".to_owned(), Json::String(env::var("DOC_DIR").unwrap_or_else(|_| datadir.clone() + "/doc/" + package)));
    ctx.insert("unitdir".to_owned(), Json::String(env::var("UNIT_DIR").unwrap_or_else(|_| libdir.clone() + "/systemd/system")));

    ctx.insert("stale_stamps".to_owned(), Json::String(env::var("STALE_STAMPS").unwrap_or_else(|_| libdir.clone() + package + "remove-stale-stamps")));

    ctx.insert("libdir".to_owned(), Json::String(libdir));
    ctx.insert("datadir".to_owned(), Json::String(datadir));
    ctx.insert("prefix".to_owned(), Json::String(prefix));

    ctx.insert("statedir".to_owned(), Json::String(env::var("STATE_DIR").unwrap_or_else(|_| "/var/spool/cron".to_owned())));

    ctx.insert("runparts".to_owned(), Json::String(env::var("RUN_PARTS").unwrap_or_else(|_| "/usr/bin/run-parts".to_owned())));

    let mut schedules = Vec::new();
    if env::var("CARGO_FEATURE_SCHED_BOOT").is_ok() {
        schedules.push(Json::String("boot".to_owned()));
    }
    if env::var("CARGO_FEATURE_SCHED_HOURLY").is_ok() {
        schedules.push(Json::String("hourly".to_owned()));
    }
    if env::var("CARGO_FEATURE_SCHED_DAILY").is_ok() {
        schedules.push(Json::String("daily".to_owned()));
    }
    if env::var("CARGO_FEATURE_SCHED_WEEKLY").is_ok() {
        schedules.push(Json::String("weekly".to_owned()));
    }
    if env::var("CARGO_FEATURE_SCHED_MONTHLY").is_ok() {
        schedules.push(Json::String("monthly".to_owned()));
    }
    if env::var("CARGO_FEATURE_SCHED_YEARLY").is_ok() {
        schedules.push(Json::String("yearly".to_owned()));
    }
    if env::var("CARGO_FEATURE_SCHED_MINUTELY").is_ok() {
        schedules.push(Json::String("minutely".to_owned()));
    }
    if env::var("CARGO_FEATURE_SCHED_QUARTERLY").is_ok() {
        schedules.push(Json::String("quarterly".to_owned()));
    }
    if env::var("CARGO_FEATURE_SCHED_SEMI_ANNUALLY").is_ok() {
        schedules.push(Json::String("semi-annually".to_owned()));
    }

    ctx.insert("schedules".to_owned(), Json::Array(schedules));

    ctx.insert("persistent".to_owned(), Json::Boolean(env::var("CARGO_FEATURE_PERSISTENT").is_ok()));

    ctx
}
