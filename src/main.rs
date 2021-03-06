extern crate rustc_serialize;
extern crate ini;
extern crate openssl;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate serde_json;

mod error;
mod header;
mod server;
mod conf;
mod payload;
mod exec;

use clap::{Arg, App};
use std::thread;
use std::sync::mpsc::channel;
use std::io::{self, Write};

macro_rules! try_log(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => {
                let _ = writeln!(&mut io::stderr(), "Error: {}", e);
                return;
            }
        }
    }}
);

fn main() {
    let matches = App::new("koukku")
                      .version("0.1.1")
                      .author("jkpl")
                      .about("Github Webhook server")
                      .arg(Arg::with_name("config")
                               .short("c")
                               .long("config")
                               .value_name("FILE")
                               .help("Configuration file location")
                               .takes_value(true)
                               .required(true))
                      .get_matches();

    let config = try_log!(matches.value_of("config")
                                 .ok_or("No config location specified"));
    start(&config);
}

fn start(config: &str) {
    let _ = try_log!(env_logger::init());
    let conf = try_log!(conf::Conf::from_file(config));
    let server = conf.server.clone();
    let threads = conf.threads;
    let projects = conf.projects.clone();

    let (tx, rx) = channel();
    let executor = exec::Executor::new(conf, rx);

    info!("Starting koukku server");

    thread::spawn(move || executor.start());

    let _ = try_log!(server::start(&server, threads, projects, tx));
}
