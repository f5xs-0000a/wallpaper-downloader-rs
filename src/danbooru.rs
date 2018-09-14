use sekibanki::Actor;
use sekibanki::ContextImmutHalf;
use std::collections::hash_map::HashMap;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use std::ops::Not;

use rating::Rating;
use config::Config;
use image_dl::ImageDownloader;
use timer::TimerMutex;

////////////////////////////////////////////////////////////////////////////////

pub struct Danbooru {
    timer: Arc<TimerMutex>, // TODO: use nazrin in the future
    client: Arc<Client>,
    tags: HashMap<String, String>,
    config: Arc<Config>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct PostJSON {
    id: usize,

    image_width: usize,
    image_height: usize,

    file_url: String,
    file_ext: String,

    tags: String,
    rating: String,
}

////////////////////////////////////////////////////////////////////////////////

impl Danbooru {
    pub fn new(client: Arc<Client>, config: Arc<Config>) -> Danbooru {
        let mut tags = HashMap::new();
        tags.insert("limit".to_owned(), "128".to_owned());

        Danbooru {
            timer: Arc::new(TimerMutex::new(Duration::new(1, 1))),
            client,
            tags,
            config,
        }
    }

    fn page_request(&mut self, page: usize, ctx: &ContextImmutHalf<Self>) {
        use tokio_threadpool::blocking;

        // generate the request form
        let mut request = self.client.get("https://danbooru.donmai.us/posts.json");

        // temporarily place the pages into the tags
        self.tags.insert("page".to_owned(), format!("{}", page));

        // place the json payload
        request.json(&self.tags);

        // build the request
        let request = request.build()
            .expect("Unexpected error while building the request.");

        // then remove the pages
        self.tags.remove("page");

        // generate the response
        let response = {
            // try to acquire the lock and, at the same time, set the thread
            // state to blocking
            let _ = blocking(|| self.timer.lock());
            self.client.execute(request)
                .expect("Error occurred when executing request.")
            
            // the lock is dropped here, allowing it to be reclaimed by someone else
        }.json::<Vec<PostJSON>>()
        .unwrap();

        // for every post in the search, try to find acceptable wallpapers
        response.into_iter()
            // the post should have an aspect ratio within tolerance
            .filter(|post| {
                self.config
                    .is_tolerated_aspect_ratio(post.image_width, post.image_height)
            })

            // the post should be within the rating standard
            .filter(|post| {
                let rating = match post.rating.as_str() {
                    "s" => Rating::Safe,
                    "q" => Rating::Questionable,
                    "e" => Rating::Explicit,
                    _ => unreachable!(),
                };

                self.config
                    .allowed_rating
                    .allows(&rating)
            })

            // the post shouldn't be animated
            .filter(|post| {
                // split the tags string by spaces, then iterate through them,
                // trying to find the "animated" tag

                post.tags
                    .split_whitespace()
                    .any(|tag| tag == "animated")

                    // since this will return true if the tag "animated" is
                    // found, we must negate that so the iterator skips the post
                    // that has the tag
                    .not()
            })

            // create an actor for each of the accepted links so that the image
            // may be downloaded
            .for_each(|post| {
                // create the image downloader
                ImageDownloader::new(
                    post.file_url,
                    format!("danbooru {}.{}", post.id, post.file_ext),
                    self.config.clone(),
                    self.client.clone(),
                    self.timer.clone(),
                )
                    // then start the actor
                    .start_actor(Default::default(), ctx.threadpool().clone());

                // then promptly drop the address to the actor. it will still
                // execute its function although it will be dropped once its
                // function has ended
            });
    }
}

impl Actor for Danbooru {
    fn on_start(&mut self, ctx: &ContextImmutHalf<Self>) {
        // TODO: actually, shouldn't be like this.
        // you need the actor to notify itself that it should perform the next
        // page so that it may be able to intercept messages from other places
        // too

        // start the loop
        for page in 1.. {
            self.page_request(page, ctx);
        }
    }
}

