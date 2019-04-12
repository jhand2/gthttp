extern crate clap;
extern crate ctrlc;

pub mod arp;
pub mod net;

use arp::*;

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use clap::{App, Arg};

struct Arguments {
    interface: String,
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
}

fn parse_args() -> Arguments {
    let args = App::new("gthttp")
        .arg(Arg::with_name("interface")
             .value_name("INTERFACE")
             .required(true)
             .help("Network interface on which to intercept traffic"))
        .arg(Arg::with_name("source_ip")
             .value_name("SOURCE_IP")
             .required(true)
             .help("Source IPv4 Address"))
        .arg(Arg::with_name("target_ip")
             .value_name("TARGET_IP")
             .required(true)
             .help("Target IPv4 Address"))
        .get_matches();

    let (iface, source_ip, target_ip) = {
        let iface = args.value_of("interface").unwrap().into();
        let sip = match args.value_of("source_ip").unwrap().parse().unwrap() {
            IpAddr::V4(v4) => v4,
            _ => panic!("source_ip is not a valid IPv4 address")
        };

        let tip = match args.value_of("target_ip").unwrap().parse().unwrap() {
            IpAddr::V4(v4) => v4,
            _ => panic!("target_ip is not a valid IPv4 address")
        };

        (iface, sip, tip)
    };

    Arguments {
        interface: iface,
        source_ip: source_ip,
        target_ip: target_ip
    }
}

fn main() {
    let args = parse_args();

    let spoofer = Arc::new(ArpSpoofer::new(&args.interface, args.source_ip, args.target_ip));

    let s = spoofer.clone();
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C!");
        s.restore_arp_state();
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    spoofer.spoof();
}
