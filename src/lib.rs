//! A library that execute functions asynchronously in another thread
//!
//! This crate provides a `join` function that will execute the given
//! closure in a new created thread, and wait for the result asynchronously
//! in coroutine context.
//!
//! If a panic happened in the given closure, the panic data will be
//! resume_unwind in the current context, so you can catch it just like
//! call a normal function
//!
//!
//! # Examples
//!
//! ```no_run
//! #[macro_use]
//! extern crate may;
//! extern crate may_thread;
//!
//! use may_thread::join;
//!
//! fn main() {
//!     let j = go!(|| {
//!         let _result = join(|| {
//!             // ......
//!         });
//!     });
//!     j.join();
//! }
//! ```
//!

#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/may_thread/0.1")]

mod pool;

use std::mem;
use std::panic;
use std::thread;

use may::coroutine;
use may::sync::Blocker;

pub use pool::ThreadPool;

/// Execute the given closure in a new created thread, and wait for the
/// result asynchronously in coroutine context.
///
/// If a panic happened in the given closure, the panic data will be
/// `resume_unwind` in the current context, so you can catch it just like
/// call a normal function
///
///
/// # Examples
///
/// ```no_run
/// #[macro_use]
/// extern crate may;
/// extern crate may_thread;
///
/// use may_thread::join;
///
/// fn main() {
///     let j = go!(|| {
///         let _result = join(|| {
///             // ......
///         });
///     });
///     j.join();
/// }
/// ```
///
pub fn join<'a, F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'a,
    T: Send,
{
    let blocker = Blocker::current();
    let mut ret = None;

    let _join = unsafe {
        spawn_unsafe(|| {
            ret = Some(panic::catch_unwind(panic::AssertUnwindSafe(f)));
            blocker.unpark();
        })
    };
    // we can't use the `join()` API here, it will block the thread!
    // we need catch the panic inside `f`, or we may wait forever!
    match blocker.park(None) {
        Ok(_) => match ret.expect("ret not set") {
            Ok(ret) => ret,
            Err(panic) => panic::resume_unwind(panic),
        },
        Err(_) => {
            // impossible be a timeout err
            // cancel happened, we do nothing here
            coroutine::trigger_cancel_panic();
        }
    }
}

/// Like `thread::spawn`, but without the closure bounds.
unsafe fn spawn_unsafe<'a, F>(f: F) -> thread::JoinHandle<()>
where
    F: FnOnce() + Send + 'a,
{
    let closure: Box<dyn FnOnce() + 'a> = Box::new(f);
    let closure: Box<dyn FnOnce() + Send> = mem::transmute(closure);
    thread::spawn(move || closure())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        let mut i = 0;
        join(|| i = 10);
        assert_eq!(i, 10);
    }

    #[test]
    #[should_panic]
    fn simple_panic() {
        join(|| assert_eq!(true, false));
    }

    #[test]
    fn catch_panic() {
        let panic = panic::catch_unwind(|| join(|| panic!("panic")));
        assert_eq!(panic.is_err(), true);
    }
}
