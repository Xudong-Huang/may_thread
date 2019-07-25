use std::net::ToSocketAddrs;

lazy_static::lazy_static! {
    static ref POOL: may_thread::ThreadPool<()> = may_thread::ThreadPool::new(||{}, 4);
}

fn main() {
    // async resolve the socket address
    may::go!(|| {
        let addr = POOL.join(|_| ("www.baidu.com", 80).to_socket_addrs().unwrap());
        for a in addr {
            println!("addr={:?}", a);
        }
    })
    .join()
    .unwrap();
}
