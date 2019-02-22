extern crate arpspoofr;

use arpspoofr::*;

fn main() {
    let (tx, mut rx) = open_interface("lo");

    loop {
        let packet = rx.next();
        println!("Got a packet!");
    }
}
