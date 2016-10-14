extern crate clap;
extern crate env_logger;
extern crate botdetector;

use clap::{Arg, App, SubCommand};

const ARG_LISTENING_ADDRESS: &'static str = "listening-address";
const ARG_LISTENING_ADDRESS_DEFAULT: &'static str = "localhost";
const ARG_LISTENING_PORT: &'static str = "listeningport";
const ARG_LISTENING_PORT_DEFAULT: &'static str = "8080";

const ARG_BACKEND_ADDRESS: &'static str = "backend-address";
const ARG_BACKEND_ADDRESS_DEFAULT: &'static str = "localhost";
const ARG_BACKEND_PORT: &'static str = "backend-port";
const ARG_BACKEND_PORT_DEFAULT: &'static str = "9000";

const ARG_FILTERFILE: &'static str = "filterfile";
const ARG_FILTERFILE_DEFAULT: &'static str = ".filter";
const ARG_LOGFILE_ACCESS: &'static str = "accesslog";

pub enum ProgArgs {
    Deploy {
        listening_address: String,
        listening_port: u16,
        backend_address: String,
        backend_port: u16,
    },
    Train,
    None,
}

fn parse_args() -> ProgArgs {

    let matches = App::new("Botdetector")
        .version("0.0.1")
        .author("Anders Bennehag")
        .about("Proof-of-concept of reverse-proxy filtering out likely \
                bots and scrapers")
        .subcommand(
            SubCommand::with_name("deploy")
                .about("Deploy a trained filter, acting as reverse proxy")
                .arg(Arg::with_name(ARG_FILTERFILE)
                     .help("A file produced in training")
                     .required(true)
                     .index(1))
                .arg(Arg::with_name(ARG_LISTENING_ADDRESS)
                        .default_value(ARG_LISTENING_ADDRESS_DEFAULT)
                        .short("a")
                        .takes_value(true)
                        .help("Interface to listen on"))
                .arg(Arg::with_name(ARG_LISTENING_PORT)
                        .default_value(ARG_LISTENING_PORT_DEFAULT)
                        .short("p")
                        .takes_value(true)
                        .help("Listening port"))
                .arg(Arg::with_name(ARG_BACKEND_ADDRESS)
                        .default_value(ARG_BACKEND_ADDRESS_DEFAULT)
                        .short("b")
                        .takes_value(true)
                        .help("Interface backend listens on"))
                .arg(Arg::with_name(ARG_BACKEND_PORT)
                        .default_value(ARG_BACKEND_PORT_DEFAULT)
                        .short("q")
                        .takes_value(true)
                        .help("Backend port"))
        )
        .subcommand(
            SubCommand::with_name("train")
                .about("Train a filter on nginx-logs")
                .arg(Arg::with_name(ARG_LOGFILE_ACCESS)
                     .help("Access-logfile from nginx")
                     .index(1))
                .arg(Arg::with_name(ARG_FILTERFILE)
                        .short("p")
                        .takes_value(true)
                        .default_value(ARG_FILTERFILE_DEFAULT)
                        .help("Listening port"))
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("deploy") {
        ProgArgs::Deploy {
            listening_port: matches.value_of(ARG_LISTENING_PORT)
                               .unwrap()
                               .parse::<u16>()
                               .unwrap(),
            listening_address: matches.value_of(ARG_LISTENING_ADDRESS)
                                      .unwrap()
                                      .to_string(),
            backend_port: matches.value_of(ARG_BACKEND_PORT)
                                   .unwrap()
                                   .parse::<u16>()
                                   .unwrap(),
            backend_address: matches.value_of(ARG_BACKEND_ADDRESS)
                                      .unwrap()
                                      .to_string(),
        }
    } else if let Some(_) = matches.subcommand_matches("train") {
        ProgArgs::Train
    } else {
        ProgArgs::None
    }
}

fn main() {
    env_logger::init().unwrap();
    match parse_args() {
        ProgArgs::Deploy {listening_port: lstn_prt, 
                          listening_address: lstn_addr, 
                          backend_port: bcknd_prt, 
                          backend_address: bcknd_addr } => {
            println!("Running botdetector-deploy");
            botdetector::server::run(&lstn_addr,
                                     lstn_prt,
                                     &bcknd_addr,
                                     bcknd_prt);
        },
        ProgArgs::Train => {
            println!("Running botdetector-train");
            println!("Train is not yet implemented");
        },
        ProgArgs::None => unreachable!(),
    }
}

