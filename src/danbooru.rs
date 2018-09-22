use reqwest::Client;
use sekibanki::{
    Actor,
    ContextImmutHalf,
    Handles,
    Message,
};
use std::{
    collections::hash_map::HashMap,
    ops::Not,
    sync::Arc,
    time::Duration,
};

use util::time_now;
use config::Config;
use image_dl::ImageDownloader;
use rating::Rating;
use timer::{
    do_lock,
    TimerMutex,
};

////////////////////////////////////////////////////////////////////////////////

pub struct Danbooru {
    timer:  Arc<TimerMutex>, // TODO: use nazrin in the future
    client: Arc<Client>,
    tags:   HashMap<String, String>,
    config: Arc<Config>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostJSON {
    id: usize,

    image_width:  usize,
    image_height: usize,

    file_url: Option<String>,
    file_ext: Option<String>,

    tag_string: String,
    rating:     String,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SearchPageNo(pub usize);

////////////////////////////////////////////////////////////////////////////////

impl Danbooru {
    pub fn new(
        client: Arc<Client>,
        config: Arc<Config>,
    ) -> Danbooru {
        let mut tags = HashMap::new();
        tags.insert("limit".to_owned(), "1000".to_owned());
        tags.insert("tags".to_owned(), "touhou -rating:e".to_owned());

        Danbooru {
            timer: Arc::new(TimerMutex::new(Duration::new(1, 1))),
            client,
            tags,
            config,
        }
    }

    fn process_post_list(
        &self,
        post_list: impl Iterator<Item = PostJSON>,
        ctx: &ContextImmutHalf<Self>,
    ) {
        // for every post in the search, try to find acceptable wallpapers
        post_list
            // there must be an available URL
            .filter(|post| {
                post.file_url.is_some() &&
                post.file_ext.is_some()
            })

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

                post.tag_string
                    .split_whitespace()
                    .any(|tag| tag == "animated")

                    // since this will return true if the tag "animated" is
                    // found, we must negate that so the iterator skips the post
                    // that has the tag
                    .not()
            })

            // create an actor for each of the accepted links so that the image
            // may be downloaded
            .for_each(|mut post| {
                // create the image downloader
                ImageDownloader::new(
                    post.file_url.take().unwrap(),
                    format!("danbooru {}.{}", post.id, post.file_ext.take().unwrap()),
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

    fn page_request(
        &mut self,
        page: usize,
        ctx: &ContextImmutHalf<Self>,
    ) {
        let url = "https://danbooru.donmai.us/posts.json";

        // generate the request form
        let mut request = self.client.get(url);

        // temporarily place the pages into the tags
        self.tags.insert("page".to_owned(), format!("{}", page));

        // place the json payload
        request.json(&self.tags);

        // then remove the pages
        self.tags.remove("page");

        println!("[{}] Attempting to download from {}, page {}", time_now(), url, page);

        // generate the response
        let response = {
            // try to acquire the lock
            let _ = do_lock(&self.timer);

            request.send()

            // the lock is dropped here, allowing it to be reclaimed by someone
            // else
        };
        
        // catch whatever happens in the requesting
        let response = match response {
            Ok(mut res) => res.json::<Vec<PostJSON>>(),
            Err(e) => {
                println!("[{}] Error processing the response: {:?}", time_now(), e);

                // retry downloading the same page
                ctx.notify(SearchPageNo(page));
                return;
            }
        };

        // catch whatever happens in the deserialization
        let response = match response {
            Ok(res) => res,
            Err(e) => {
                println!("[{}] Error deserializing JSON text: {:?}", time_now(), e);

                // retry downloading the same page
                ctx.notify(SearchPageNo(page));
                return;
            }
        };

        // The query finishes if danbooru has no more posts returned.
        // TODO: not always
        if response.is_empty() {
            println!("[{}] Finished! No more posts to get!", time_now());
            return;
        }

        self.process_post_list(response.into_iter(), ctx);

        // notify self to perform the next page
        ctx.notify(SearchPageNo(page + 1))
    }
}

impl Actor for Danbooru {
    fn on_start(
        &mut self,
        ctx: &ContextImmutHalf<Self>,
    ) {
    }
}

impl Handles<SearchPageNo> for Danbooru {
    type Response = ();

    fn handle(
        &mut self,
        msg: SearchPageNo,
        ctx: &ContextImmutHalf<Self>,
    ) -> Self::Response {
        self.page_request(msg.0, ctx);
    }
}
