extern crate reqwest;
extern crate cursive;
extern crate parking_lot;

////////////////////////////////////////////////////////////////////////////////

use reqwest::Client;
use std::sync::Arc;
use std::collections::hash_map::HashMap;
use parking_lot::Mutex;
use std::time::Duration;
use parking_lot::MutexGuard;
use std::time::Instant;

////////////////////////////////////////////////////////////////////////////////

fn do_it(client: Arc<Client>) {
    // create the timelock
    let danbooru_timelock = TimerMutex::new(Duration::new(1, 0));

    // create the payload
    let mut map = HashMap::new();
    map.insert("page", "1");
    map.insert("limit", "128");
    map.insert("tags", "-comic rating:safe");

    // create the request
    let mut request = client.get("https://danbooru.donmai.us/posts.json");
    request.json(&map);

    // generate the response
    let response_text = {
        let _ = danbooru_timelock.lock();
        request.send()

        // the lock is dropped here, allowing it to be reclaimed by someone else
    }
        .expect("Error occured when requesting for data.")
        .text()
        .expect("Text can't be unwrapped.");

    println!("{}", response_text);
}

fn main() {
    // create the client
    let client = Arc::new(Client::new());

    // create the threadpool

    do_it(client);

    // user, through curses, can modify the properties being used
    
    // site manager reads through the properties, iterates through the list, and
    // spawns a unit executable that processes each accepted link
    
    // unit executable attempts to download the payload and exits promptly
}

pub struct TimerMutex {
    duration: Duration,

    // the bool also states whether this has been attempted to lock on or not
    lock: Mutex<bool>,
}

pub struct TimerLock<'a>(MutexGuard<'a, bool>);

impl TimerMutex {
    pub fn new(duration: Duration) -> TimerMutex {
        TimerMutex {
            duration,
            lock: Mutex::new(false),
        }
    }

    pub fn lock<'a>(&'a self) -> TimerLock<'a> {
        use std::thread::sleep;

        // acquire the lock; the lock can't be acquired if someone else has
        // acquired it
        let mut lockguard = self.lock.lock();

        if *lockguard {
            sleep(self.duration);
        }

        TimerLock(lockguard)
    }
}
