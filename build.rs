extern crate rumblebars;
extern crate rustc_serialize;

use std::env;
use std::path::Path;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::collections::BTreeMap;

use rumblebars::{Template, EvalContext};
use rustc_serialize::json::Json;

static UNITS_DIR: &'static str = "units";
static MAN_DIR: &'static str = "man";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let output = Path::new(&out_dir);

    let data = build_render_data();

    let mut config = File::create(out_dir.clone() + "/config.rs").unwrap();
    writeln!(config, "pub static USERS_CRONTAB_DIR: &'static str = {:?};", data["statedir"].as_string().unwrap()).unwrap();
    writeln!(config, "pub static PACKAGE: &'static str = {:?};", data["package"].as_string().unwrap()).unwrap();
    writeln!(config, "pub static BIN_DIR: &'static str = {:?};", data["bindir"].as_string().unwrap()).unwrap();
    writeln!(config, "pub static LIB_DIR: &'static str = {:?};", data["libdir"].as_string().unwrap()).unwrap();

    let data = Json::Object(data);

    compile_templates(UNITS_DIR, output, &data);
    compile_templates(MAN_DIR, output, &data);
}

fn compile_templates<P: AsRef<Path>>(source_dir: &str, output_dir: P, data: &Json) {
    let ctx = EvalContext::new();
    for entry in fs::read_dir(source_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        if name.ends_with(".in") {
            let target = output_dir.as_ref().join(&name[..name.len()-3]);
            let tmpl = File::open(entry.path()).and_then(|mut file| {
                let mut buf = String::new();
                file.read_to_string(&mut buf).map(|_| Template::new(&*buf).unwrap())
            }).unwrap();

            println!("compiling from template: {:?} -> {:?}...", entry.path(), target);
            tmpl.eval(data, &mut File::create(target).unwrap(), &ctx).unwrap();
        }
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

    ctx.insert("libdir".to_owned(), Json::String(libdir));
    ctx.insert("datadir".to_owned(), Json::String(datadir));
    ctx.insert("prefix".to_owned(), Json::String(prefix));

    ctx.insert("statedir".to_owned(), Json::String(env::var("STATE_DIR").unwrap_or_else(|_| "/var/spool/cron".to_owned())));

    ctx.insert("runparts".to_owned(), Json::String(env::var("RUN_PARTS").unwrap_or_else(|_| "/usr/bin/run-parts".to_owned())));

    ctx.insert("persistent".to_owned(), Json::Boolean(env::var("CARGO_FEATURE_PERSISTENT").is_ok()));

    ctx
}

