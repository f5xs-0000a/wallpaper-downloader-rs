use reqwest::Client;
use sekibanki::{
    Actor,
    ContextImmutHalf,
};
use std::{
    fs::File,
    sync::Arc,
};

use config::Config;
use timer::{
    do_lock,
    TimerMutex,
};

////////////////////////////////////////////////////////////////////////////////

pub struct ImageDownloader {
    url:      String,
    filename: String,
    config:   Arc<Config>,
    client:   Arc<Client>,
    timer:    Arc<TimerMutex>,
}

impl ImageDownloader {
    pub fn new(
        url: String,
        filename: String,
        config: Arc<Config>,
        client: Arc<Client>,
        timer: Arc<TimerMutex>,
    ) -> ImageDownloader {
        ImageDownloader {
            url,
            filename,
            config,
            client,
            timer,
        }
    }
}

impl Actor for ImageDownloader {
    fn on_start(
        &mut self,
        _ctx: &ContextImmutHalf<Self>,
    ) {
        // generate the request
        let request = self.client.get(self.url.as_str()).build().unwrap();

        println!("Attempting to download from {}", request.url());

        // generate the response
        let mut response = {
            // try to acquire the lock
            let _ = do_lock(&self.timer);

            self.client
                .execute(request)
                .expect("Error occurred when executing request.")

            // the lock is dropped here, allowing it to be reclaimed by someone
            // else
        };

        // create the file
        let filepath = self.config.location.join(&self.filename);
        // ignore the error for this one; it may have already been created
        File::create(filepath.clone());

        // open the file
        let mut file = File::open(filepath).unwrap();

        // write to the file
        response.copy_to(&mut file);

        // promptly kill thyself
    }
}
