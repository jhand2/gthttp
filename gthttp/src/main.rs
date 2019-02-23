extern crate arpspoofr;

use arpspoofr::*;

fn main() {
    let (_tx, mut rx) = open_interface("lo");

    loop {
        let _packet = rx.next();
        println!("Got a packet!");
    }
}
