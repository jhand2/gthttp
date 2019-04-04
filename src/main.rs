extern crate clap;
extern crate ctrlc;

pub mod arp;
pub mod net;

use arp::*;
use net::*;

//use pnet::transport::*;

use std::net::{IpAddr, Ipv4Addr};
use std::thread;
use std::sync::Arc;
use clap::{App, Arg};

use pnet::packet::Packet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;

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

    let s1 = spoofer.clone();
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C!");
        s1.restore_arp_state();
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // Spoof arp responses so we can receive traffic for the source and target
    let s2 = spoofer.clone();
    let _arp_spoof_thread = thread::spawn(move || {
        s2.spoof();
    });

    // Receive intercepted packets
    // TODO: Refactor this to somewhere else. Basically we just want to set up and SSL proxy that
    // will create one trusted connection with the client and one trusted connection with the
    // server
    let (_tx, mut rx, _) = open_datalink_if(&args.interface);
    loop {
        match rx.next() {
            Ok(data) => {
                let ethernet_packet = EthernetPacket::new(data).unwrap();
                let ethernet_payload = ethernet_packet.payload();

                match ethernet_packet.get_ethertype() {
                    EtherTypes::Ipv4 => {
                        let ip_pkt = Ipv4Packet::new(ethernet_payload).unwrap();
                        let src = ip_pkt.get_source();
                        let dst = ip_pkt.get_destination();
                        println!("src {}, dst {}", src, dst);
                    },
                    _ => continue
                }
            },
            Err(e) => panic!("An error occurred while reading: {}", e)
        }
    }

    //let _res = arp_spoof_thread.join();
}
