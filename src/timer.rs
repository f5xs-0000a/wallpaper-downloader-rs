use parking_lot::{
    Mutex,
    MutexGuard,
};
use std::time::{
    Duration,
    Instant,
};
use tokio_threadpool::blocking;

///////////////////////////////////////////////////////////////////////////////

pub struct TimerMutex {
    duration: Duration,

    // the bool also states whether this has been attempted to lock on or not
    lock: Mutex<Option<Instant>>,
}

pub struct TimerLock<'a>(MutexGuard<'a, Option<Instant>>);

////////////////////////////////////////////////////////////////////////////////

impl TimerMutex {
    pub fn new(duration: Duration) -> TimerMutex {
        TimerMutex {
            duration,
            lock: Mutex::new(None),
        }
    }

    pub fn lock<'a>(&'a self) -> TimerLock<'a> {
        use std::thread::sleep;

        // acquire the lock; the lock can't be acquired if someone else has
        // acquired it
        let lockguard = self.lock.lock();

        println!("Lock acquired!");

        // find the amount of time it would take before it would be
        // `self.duration` seconds after the last lock
        if let Some(ref last_release) = *lockguard {
            // past ---|-----------------|--- future
            //         |                 |
            //         +-- last_release  +-- now()
            //         |                 |
            //         +-----------------+
            //            duration_since

            // find the amount of time that has elapsed since last release
            let duration_since =
                Instant::now().duration_since(last_release.clone());

            if let Some(sleep_dur) = self.duration.checked_sub(duration_since) {
                // sleep for the remaining time
                sleep(sleep_dur);
            }
        }

        TimerLock(lockguard)
    }
}

impl<'a> Drop for TimerLock<'a> {
    fn drop(&mut self) {
        // record the time the lock was dropped
        *self.0 = Some(Instant::now());

        println!("Lock released!");
    }
}

#[inline]
pub fn do_lock<'a>(mutex: &'a TimerMutex) -> TimerLock<'a> {
    let async = blocking(|| mutex.lock())
        .expect("timer::do_lock() must be called if the calling thread is on a ThreadPool.");

    if let ::futures::Async::Ready(lock) = async {
        lock
    }

    else {
        panic!("Maximum number of blocking threads reached!. You may want to consider increasing this.");
    }
}
