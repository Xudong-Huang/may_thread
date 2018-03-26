//! thread pool that share the same work queue
//! when init thread pool, we need to tell it how to init the associated data
//! ThreadPool

use std::thread;
use may::coroutine;
use may::sync::mpmc;

#[doc(hidden)]
trait FnBox<S> {
    fn call_box(self: Box<Self>, state: &mut S);
}

impl<S, F: FnOnce(&mut S)> FnBox<S> for F {
    fn call_box(self: Box<Self>, state: &mut S) {
        (*self)(state)
    }
}

/// Thread pool that can run closures in parallel
pub struct ThreadPool<S> {
    // all worker thread share the same mpmc queue
    // used to push works into the queue
    queue_tx: mpmc::Sender<Box<FnBox<S> + Send>>,

    // thread pool handles
    threads: Vec<Option<thread::JoinHandle<()>>>,
}

unsafe impl<S> Send for ThreadPool<S> {}
unsafe impl<S> Sync for ThreadPool<S> {}

// S should be created in thread in parallel
// thus not need to be Send, but that need f to be Send and Sync
impl<S: Send + 'static> ThreadPool<S> {
    /// create a thread pool with the specified state initializer and pool size
    pub fn new<F>(f: F, size: usize) -> Self
    where
        F: Fn() -> S,
    {
        let mut threads = Vec::with_capacity(size);
        let (tx, rx) = mpmc::channel::<Box<FnBox<S> + Send>>();
        for _i in 0..size {
            // each thread has a internal state
            let mut state = f();
            let rx = rx.clone();
            let thread = thread::spawn(move || {
                for work in rx.into_iter() {
                    // execute the work
                    work.call_box(&mut state);
                }
            });
            threads.push(Some(thread));
        }

        ThreadPool {
            queue_tx: tx,
            threads,
        }
    }

    /// execute a closure by the thread pool
    /// different from the spawn method in that we have to wailt until it returns
    pub fn join<'a, F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut S) -> T + Send + 'a,
        T: Send,
    {
        use std::mem;
        use std::panic;

        let mut ret = unsafe { mem::zeroed() };
        {
            let clo: Box<FnBox<S> + Send> = Box::new(|s: &mut S| {
                ret = panic::catch_unwind(panic::AssertUnwindSafe(|| f(s)));
            });
            let clo: Box<FnBox<S> + Send + 'static> = unsafe { mem::transmute(clo) };
            self.queue_tx
                .send(clo)
                .expect("failed to send to work queue");
            coroutine::sleep(::std::time::Duration::from_secs(1));
        }

        ret.unwrap()
    }
}

impl<S> Drop for ThreadPool<S> {
    fn drop(&mut self) {
        // first need to destroy the tx side so that others will return
        // just substitude with a dummy one
        let (tx, _) = mpmc::channel();
        self.queue_tx = tx;

        // wait all the worker returns
        for thread in self.threads.iter_mut() {
            thread.take().map(|t| t.join().ok());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_pool() {
        let pool = ThreadPool::new(|| 0, 4);
        let a = pool.join(|s| {
            *s += 1;
            *s
        });
        assert_eq!(a, 1);
    }
}
