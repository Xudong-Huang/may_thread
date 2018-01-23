# may_thread

Execute functions asynchronously in another thread. Mainly designed for use in coroutine context.

Some APIs would block the thread execution, but hard to implement an async version, e.g. the DNS resolve. We can put those blocking API execution in another thread and asynchronously wait for the result.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

