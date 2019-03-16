extern crate clap;
extern crate arpspoofr;

use arpspoofr::*;

use std::net::IpAddr;
use clap::{App, Arg};

fn main() {
    let matches = App::new("gthttp")
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

    // TODO: handle case where interface is not valid
    let (mut tx, mut rx, iface) = open_interface(matches.value_of("interface").unwrap());

    let local_mac = iface.mac_address();
    let local_ip = iface.ips.first().unwrap().ip();

    println!("My IP: {}", local_ip);
    
    if let IpAddr::V4(local_ipv4) = local_ip {
        // TODO: Take in as input
        let source_ip_res = matches.value_of("source_ip").unwrap().parse();
        let target_ip_res = matches.value_of("target_ip").unwrap().parse();

        // TODO: Nested if lets suck. Find a better way
        if let Ok(source_ip) = source_ip_res {
            if let Ok(target_ip) = target_ip_res {
                // Lookup mac for source and target
                let source_mac = lookup_arp(&mut *tx, &mut *rx, local_ipv4, local_mac, source_ip);
                println!("Source MAC: {}", source_mac);

                let target_mac = lookup_arp(&mut *tx, &mut *rx, local_ipv4, local_mac, target_ip);
                println!("Target MAC: {}", target_mac);

                // Send packet to bind source ip to local mac
                //send_arp(&mut *tx, source_ip, local_mac, target_ip, target_mac);

                // Send packet to bind target ip to local mac
                //send_arp(&mut *tx, target_ip, local_mac, source_ip, source_mac);
            }
        }
    }
}
