pub struct ImageDownloader {
    url: String,
    filename: String,
    config: Arc<Config>,
    client: Arc<Client>,
    timer: Arc<TimerMutex>,
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
    fn on_start(&mut self, _ctx: &ContextImmutHalf) {
        use std::iter::once;
        use tokio_threadpool::blocking;

        // generate the request
        let mut request = self.client.get("https://danbooru.donmai.us/posts.json").build();

        // generate the response
        let response = {
            // try to acquire the lock and, at the same time, set the thread
            // state to blocking
            let _ = blocking(|| self.timer.lock());
            self.client.execute(request)
                .expect("Error occurred when executing request.")
            
            // the lock is dropped here, allowing it to be reclaimed by someone
            // else
        };

        // create the file
        let filepath = config.location.join(self.filename);
        // ignore the error for this one; it may have already been created
        File::create(filepath);

        // open the file
        let file = File::open(filepath).unwrap();

        // write to the file
        response.copy_to(file);

        // promptly kill thyself
    }
}
