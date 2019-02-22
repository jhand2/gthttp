use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface, DataLinkSender, DataLinkReceiver};

pub fn open_interface(interface: &str)
        -> (Box<DataLinkSender>, Box<DataLinkReceiver>) {
    
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

    return (tx, rx)
}
