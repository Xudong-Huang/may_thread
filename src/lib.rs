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
//!         // resolve the dns in another thread
//!         let _addr = join(|| {});
//!     });
//!     j.join();
//! }
//! ```
//!

#![warn(missing_debug_implementations)]
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/may_thread/0.1")]

#[doc(hiden)]
extern crate may;

use std::mem;
use std::panic;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use may::coroutine;
use may::sync::{AtomicOption, Blocker};

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
///         // resolve the dns in another thread
///         let _addr = join(|| {});
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
    let ret = Arc::new(AtomicOption::none());
    let err = Arc::new(AtomicOption::none());

    let _join = unsafe {
        let ret = ret.clone();
        let err = err.clone();
        let blocker = blocker.clone();
        spawn_unsafe(move || {
            let exit = panic::catch_unwind(panic::AssertUnwindSafe(f));
            match exit {
                Ok(r) => {
                    ret.swap(r, Ordering::Relaxed);
                }
                Err(e) => {
                    err.swap(e, Ordering::Relaxed);
                }
            }
            blocker.unpark();
        })
    };
    // we can't use the `join()` API here, it will block the thread!
    // we need catch the panic inside `f`, or we may wait forever!
    match blocker.park(None) {
        Ok(_) => match ret.take(Ordering::Relaxed) {
            Some(v) => v,
            None => match err.take(Ordering::Relaxed) {
                Some(panic) => panic::resume_unwind(panic),
                None => panic!("failed to get result"),
            },
        },
        Err(_) => {
            // impossible be a timeout err
            // cancel happened, we do nothing here
            coroutine::trigger_cancel_panic();
        }
    }
}

#[doc(hidden)]
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<Self>) {
        (*self)()
    }
}

/// Like `thread::spawn`, but without the closure bounds.
unsafe fn spawn_unsafe<'a, F>(f: F) -> thread::JoinHandle<()>
where
    F: FnOnce() + Send + 'a,
{
    let closure: Box<FnBox + 'a> = Box::new(f);
    let closure: Box<FnBox + Send> = mem::transmute(closure);
    thread::spawn(move || closure.call_box())
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
