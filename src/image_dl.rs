use reqwest::Client;
use sekibanki::{
    Actor,
    ContextImmutHalf,
};
use std::{
    fs::File,
    io::Write,
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
        use std::fs::OpenOptions;
        // generate the request
        let mut request = self.client.get(self.url.as_str());

        println!("Attempting to download from {}", self.url);

        // generate the response
        let mut response = {
            // try to acquire the lock
            let _ = do_lock(&self.timer);

            let response = request
                .send()
                .expect("Error occurred when executing request.");

            println!("{}", response.status());

            response

            // the lock is dropped here, allowing it to be reclaimed by someone
            // else
        };

        // TODO: use a buffer that will notify ncurses about the progress next
        // time
        // TODO: copying the content to the buffer is a blocking effort. wrap it
        // in a blocking()
        let mut buffer = Vec::new();
        let result = response.copy_to(&mut buffer).unwrap();

        // create the file
        let filepath = self.config.location.join(&self.filename);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(filepath)
            .unwrap();

        file.write(&*buffer);
        file.flush();

        // promptly kill thyself
        println!("Finished downloading {}", self.filename);
    }
}
