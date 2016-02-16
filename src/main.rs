extern crate ini;
extern crate crypto;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;

mod error;
mod header;
mod server;
mod conf;

use clap::{Arg, App};
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
    let matches = App::new("hubikoukku")
        .version("0.1")
        .author("jkpl")
        .about("Github Webhook server")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Configuration file location")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("server")
             .short("s")
             .long("server")
             .value_name("HOST:PORT")
             .help("The address and port to run the server on")
             .takes_value(true)
             .required(true))
        .get_matches();

    let config = try_log!(matches.value_of("config")
                          .ok_or("No config location specified"));
    let server = try_log!(matches.value_of("server")
                          .ok_or("No server address specified"));

    start(&config, &server);
}

fn start(config: &str, server: &str) {
    let _ = try_log!(env_logger::init());
    let s = try_log!(conf::Conf::from_file(config));
    info!("Starting hubikoukku server");
    let _ = try_log!(server::start(server));
}

