extern crate reqwest;
extern crate cursive;
extern crate parking_lot;
extern crate tokio_threadpool;
extern crate sekibanki;
extern crate serde;
#[macro_use] extern crate serde_derive;

////////////////////////////////////////////////////////////////////////////////

mod timer;
mod config;
mod image_dl;
mod rating;

mod danbooru;

////////////////////////////////////////////////////////////////////////////////

use sekibanki::Actor;
use reqwest::Client;
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
    danbooru::Danbooru::new(
        client.clone(),
        config.clone()
    ).start_actor(
        Default::default(),
        threadpool.sender().clone()
    );

    // unfortunately, we still don't have a proper event loop because the
    // ncurses is still not set up.
    loop {
        ::std::thread::sleep(::std::time::Duration::new(1, 0));
    }



    // user, through curses, can modify the properties being used
    
    // site manager reads through the properties, iterates through the list, and
    // spawns a unit executable that processes each accepted link
    
    // unit executable attempts to download the payload and exits promptly
}
