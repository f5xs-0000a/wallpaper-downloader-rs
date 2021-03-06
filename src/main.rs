extern crate chrono;
extern crate futures;
extern crate parking_lot;
extern crate reqwest;
extern crate sekibanki;
extern crate serde;
extern crate tokio_threadpool;
#[macro_use]
extern crate serde_derive;

////////////////////////////////////////////////////////////////////////////////

mod config;
mod image_dl;
mod rating;
mod timer;
mod util;

mod danbooru;

////////////////////////////////////////////////////////////////////////////////

use reqwest::Client;
use sekibanki::Actor;
use std::sync::Arc;
use tokio_threadpool::ThreadPool;

use config::Config;

////////////////////////////////////////////////////////////////////////////////

fn main() {
    // create the client
    let client = Arc::new(Client::new());

    // create the threadpool
    let threadpool = ThreadPool::new();

    // create the global config;
    let config = Arc::new(Config::default());

    // create the Danbooru main actor
    let mut danbooru = danbooru::Danbooru::new(client.clone(), config.clone())
        .start_actor(Default::default(), threadpool.sender().clone());

    danbooru.send(danbooru::Search::Start);

    // unfortunately, we still don't have a proper event loop because the
    // ncurses is still not set up.
    loop {
        ::std::thread::sleep(::std::time::Duration::new(1, 0));
    }
}
