extern crate clap;
extern crate botdetector;

use clap::{Arg, App, SubCommand};


const ARG_LISTENING_ADDRESS: &'static str = "address";
const ARG_LISTENING_ADDRESS_DEFAULT: &'static str = "localhost";
const ARG_LISTENING_PORT: &'static str = "port";
const ARG_LISTENING_PORT_DEFAULT: &'static str = "8080";
const ARG_FILTERFILE: &'static str = "filterfile";
const ARG_FILTERFILE_DEFAULT: &'static str = ".filter";
const ARG_LOGFILE_ACCESS: &'static str = "accesslog";

pub struct DetectorArgs {
    port: u16,
    address: String,
}

fn parse_args() -> DetectorArgs {

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
                     .index(1))
                .arg(Arg::with_name(ARG_LISTENING_ADDRESS)
                        .short("a")
                        .takes_value(true)
                        .default_value(ARG_LISTENING_ADDRESS_DEFAULT)
                        .help("Interface to listen on"))
                .arg(Arg::with_name(ARG_LISTENING_PORT)
                        .short("p")
                        .takes_value(true)
                        .default_value(ARG_LISTENING_PORT_DEFAULT)
                        .help("Listening port"))
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

    DetectorArgs {
        port: matches.value_of(ARG_LISTENING_PORT)
                     .unwrap()
                     .parse::<u16>()
                     .unwrap(),
        address: matches.value_of(ARG_LISTENING_ADDRESS)
                        .unwrap().to_string(),
    }
}

fn main() {
    let args = parse_args();

    botdetector::server::run(&args.address, args.port);
}

