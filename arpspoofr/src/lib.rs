use std::net::Ipv4Addr;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface, DataLinkSender, DataLinkReceiver};
use pnet::datalink::MacAddr;

use pnet::packet::ethernet::{MutableEthernetPacket, EtherTypes};
use pnet::packet::{MutablePacket, Packet};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations};
use pnet::packet::arp::MutableArpPacket;


pub fn open_interface(interface: &str)
        -> (Box<DataLinkSender>, Box<DataLinkReceiver>, NetworkInterface) {
    
    // Find interface with given name
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
                        .filter(|iface: &NetworkInterface| iface.name == interface)
                        .next()
                        .unwrap();

    // TODO: Learn how to handle errors better than panic
    let (tx, rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    return (tx, rx, interface)
}

pub fn send_arp(tx: &mut DataLinkSender,
                send_ip: Ipv4Addr,
                send_mac: MacAddr,
                rec_ip: Ipv4Addr,
                rec_mac: MacAddr) {
    let mut eth_buf = [0u8; 42];
    let mut eth_pkt = MutableEthernetPacket::new(&mut eth_buf).unwrap();

    eth_pkt.set_destination(rec_mac);
    eth_pkt.set_source(send_mac);
    eth_pkt.set_ethertype(EtherTypes::Arp);

    let mut arp_buf = [0u8; 28];
    let mut arp_pkt = MutableArpPacket::new(&mut arp_buf).unwrap();

    arp_pkt.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_pkt.set_protocol_type(EtherTypes::Ipv4);
    arp_pkt.set_hw_addr_len(6);
    arp_pkt.set_proto_addr_len(4);
    arp_pkt.set_operation(ArpOperations::Reply);
    arp_pkt.set_sender_hw_addr(send_mac);
    arp_pkt.set_sender_proto_addr(send_ip);
    arp_pkt.set_target_hw_addr(rec_mac);
    arp_pkt.set_target_proto_addr(rec_ip);

    eth_pkt.set_payload(arp_pkt.packet_mut());

    tx.send_to(eth_pkt.packet(), None);
}
