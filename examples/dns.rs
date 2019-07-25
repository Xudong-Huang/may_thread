use std::net::ToSocketAddrs;

fn main() {
    // async resolve the socket address
    may::go!(|| {
        let addr = may_thread::join(|| ("www.baidu.com", 80).to_socket_addrs().unwrap());
        for a in addr {
            println!("addr={:?}", a);
        }
    })
    .join()
    .unwrap();
}
