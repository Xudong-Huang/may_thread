# may_thread

Execute functions asynchronously in another thread. Mainly designed for use in coroutine context.

Some APIs would block the thread execution, but hard to implement an async version, e.g. the DNS resolve. We can put those blocking API execution in another thread and asynchronously wait for the result.

[![Build Status](https://travis-ci.org/Xudong-Huang/may_thread.svg?branch=master)](https://travis-ci.org/Xudong-Huang/may_thread)
[![Build status](https://ci.appveyor.com/api/projects/status/c6v7w9po819ya3yb/branch/master?svg=true)](https://ci.appveyor.com/project/Xudong-Huang/may-thread/branch/master)


## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
may_process = { git = "https://github.com/Xudong-Huang/may_thread.git" }
```

Next you can use the API directly:

```rust,no_run
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
```

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

