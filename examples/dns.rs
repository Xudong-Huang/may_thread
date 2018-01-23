#[macro_use]
extern crate may;
extern crate may_thread;

use std::net::ToSocketAddrs;

fn main() {
    // async resolve the socket address
    go!(|| {
        let addr = may_thread::join(|| ("www.baidu.com", 80).to_socket_addrs().unwrap());
        for a in addr {
            println!("addr={:?}", a);
        }
    }).join().unwrap();
}
