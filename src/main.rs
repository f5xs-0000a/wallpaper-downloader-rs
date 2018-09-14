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

mod danbooru;

////////////////////////////////////////////////////////////////////////////////

use sekibanki::Actor;
use reqwest::Client;
use std::sync::Arc;
use tokio_threadpool::ThreadPool;

////////////////////////////////////////////////////////////////////////////////


fn main() {
    // create the client
    let client = Arc::new(Client::new());

    // create the threadpool
    let threadpool = ThreadPool::new();

    // create the Danbooru main actor
    let danbooru = danbooru::Danbooru::new(client.clone()).start_actor(Default::default(), threadpool.sender().clone());



    // user, through curses, can modify the properties being used
    
    // site manager reads through the properties, iterates through the list, and
    // spawns a unit executable that processes each accepted link
    
    // unit executable attempts to download the payload and exits promptly
}
