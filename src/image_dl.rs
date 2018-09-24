use reqwest::Client;
use sekibanki::{
    Actor,
    ContextImmutHalf,
};
use std::{
    fs::{
        remove_file,
        File,
    },
    io::Write,
    sync::Arc,
};

use config::Config;
use timer::{
    do_lock,
    TimerMutex,
};
use util::time_now;

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

        println!("[{}] Attempting to download from {}", time_now(), self.url);

        // generate the response
        let response = {
            // try to acquire the lock
            let _ = do_lock(&self.timer);

            request.send()

            // the lock is dropped here, allowing it to be reclaimed by someone
            // else
        };

        let mut response = match response {
            Ok(res) => res,
            Err(e) => {
                println!(
                    "[{}] Error processing the response: {:?}",
                    time_now(),
                    e
                );
                // but do not attempt to download the image again
                // TODO: this behavior will change in the future
                return;
            },
        };

        // TODO: use a buffer that will notify ncurses about the progress next
        // time
        // TODO: copying the content to the buffer is a blocking effort. wrap it
        // in a blocking()
        let mut buffer = Vec::new();
        let result = response.copy_to(&mut buffer).unwrap();

        // create the file
        let filepath = self.config.location.join(&self.filename);

        // try to delete the file first, if it exists.
        remove_file(&filepath);

        // then create the file to be writable
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(&filepath);

        let mut file = match file {
            Ok(f) => f,
            Err(e) => {
                println!(
                    "[{}] Error opening the file {:?} for write access: {:?}",
                    time_now(),
                    filepath.to_str(),
                    e
                );
                return;
            },
        };

        file.write(&*buffer);
        file.flush();

        // promptly kill thyself
        println!("Finished downloading {}", self.filename);
    }
}
