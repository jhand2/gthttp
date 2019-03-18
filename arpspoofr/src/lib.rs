use std::net::Ipv4Addr;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface, DataLinkSender, DataLinkReceiver};
use pnet::datalink::MacAddr;

use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket, EtherTypes};
use pnet::packet::{MutablePacket, Packet};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperation, ArpOperations};
use pnet::packet::arp::{ArpPacket, MutableArpPacket};

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
                rec_mac: MacAddr,
                op: ArpOperation) {
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
    arp_pkt.set_operation(op);
    arp_pkt.set_sender_hw_addr(send_mac);
    arp_pkt.set_sender_proto_addr(send_ip);
    arp_pkt.set_target_hw_addr(rec_mac);
    arp_pkt.set_target_proto_addr(rec_ip);

    eth_pkt.set_payload(arp_pkt.packet_mut());

    tx.send_to(eth_pkt.packet(), None);
}

pub fn lookup_arp(tx: &mut DataLinkSender,
                  rx: &mut DataLinkReceiver,
                  send_ip: Ipv4Addr,
                  send_mac: MacAddr,
                  rec_ip: Ipv4Addr) -> MacAddr {
    let target_mac = MacAddr::new(255, 255, 255, 255, 255, 255);


    // 
    loop {
        send_arp(&mut *tx, send_ip, send_mac, rec_ip, target_mac, ArpOperations::Request);

        match rx.next() {
            Ok(data) => {
                let ethernet_packet = EthernetPacket::new(data).unwrap();
                let ethernet_payload = ethernet_packet.payload();
                let arp_packet = ArpPacket::new(ethernet_payload).unwrap();
                let arp_reply_op = ArpOperation::new(2_u16);

                if arp_packet.get_operation() == arp_reply_op {
                    return arp_packet.get_sender_hw_addr();
                }
            },
            Err(e) => panic!("An error occurred while reading packet: {}", e)
        }
    }
}

