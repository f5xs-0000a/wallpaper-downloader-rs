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

use config::Config;
use image_dl::ImageDownloader;
use rating::Rating;
use timer::{
    do_lock,
    TimerMutex,
};
use util::time_now;

////////////////////////////////////////////////////////////////////////////////

pub struct Danbooru {
    timer:  Arc<TimerMutex>, // TODO: use nazrin in the future
    client: Arc<Client>,
    data:   HashMap<String, String>,
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

    is_deleted: bool,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Search {
    Start,
    Under(usize),
    Above(usize),
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Direction {
    Later,
    Earlier,
}

////////////////////////////////////////////////////////////////////////////////

impl Direction {
    pub fn is_later(&self) -> bool {
        match self {
            Direction::Later => true,
            _ => false,
        }
    }

    pub fn is_earlier(&self) -> bool {
        match self {
            Direction::Earlier => true,
            _ => false,
        }
    }
}

impl Search {
    pub fn get_from_direction(&self) -> Option<(usize, Direction)> {
        match self {
            Search::Start => None,
            Search::Under(x) => Some((x.clone(), Direction::Earlier)),
            Search::Above(x) => Some((x.clone(), Direction::Later)),
        }
    }

    pub fn obtain_tags(&self) -> String {
        let mut tags = String::new();
        let allowable_tags = 2;

        let push_tag = |tags: &mut String, tag: &str| {
            tags.push_str(tag);
            tags.push(' ');
        };

        if let Some((from, direction)) = self.get_from_direction() {
            if direction.is_later() {
                push_tag(&mut tags, &format!("id:>{}", from));
                push_tag(&mut tags, "order:id_asc");
            } else {
                push_tag(&mut tags, &format!("id:<{}", from));
            }
        }

        if (&tags).split_whitespace().count() < allowable_tags {
            push_tag(&mut tags, "touhou");
        }

        if (&tags).split_whitespace().count() < allowable_tags {
            push_tag(&mut tags, "-rating:e");
        }

        tags
    }
}

impl Danbooru {
    pub fn new(
        client: Arc<Client>,
        config: Arc<Config>,
    ) -> Danbooru {
        let mut data = HashMap::new();
        data.insert("limit".to_owned(), "1000".to_owned());
        data.insert("page".to_owned(), "1".to_owned());

        Danbooru {
            timer: Arc::new(TimerMutex::new(Duration::new(1, 0))),
            client,
            data,
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

            // the post should not be deleted
            .filter(|post| {
                !post.is_deleted
            })

            // the post should have a "touhou" tag
            .filter(|post| {
                // split the tags string by spaces, then iterate through them,
                // trying to find the "animated" tag

                post.tag_string
                    .split_whitespace()
                    .any(|tag| tag == "touhou")

                    // since this will return true if the tag "animated" is
                    // found, we must negate that so the iterator skips the post
                    // that has the tag
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

    fn request(
        &mut self,
        search: Search,
        ctx: &ContextImmutHalf<Self>,
    ) {
        let url = "https://danbooru.donmai.us/posts.json";

        // generate the tags
        self.data.insert("tags".to_owned(), search.obtain_tags());

        // generate the request form
        let mut request = self.client.get(url).json(&self.data);

        println!("[{}] Attempting to download from {}", time_now(), url);

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
                println!(
                    "[{}] Error processing the response: {:?}",
                    time_now(),
                    e
                );

                // retry downloading the same page
                ctx.notify(search);
                return;
            },
        };

        // catch whatever happens in the deserialization
        let response = match response {
            Ok(res) => res,
            Err(e) => {
                println!(
                    "[{}] Error deserializing JSON text: {:?}",
                    time_now(),
                    e
                );

                // retry downloading the same page
                ctx.notify(search);
                return;
            },
        };

        // The query finishes if danbooru has no more posts returned.
        // TODO: not always
        if response.is_empty() {
            println!("[{}] Finished! No more posts to get!", time_now());
            return;
        }
        let lowest_id = response
            .iter()
            .min_by(|lpost, rpost| lpost.id.cmp(&rpost.id))
            .unwrap()
            .id
            .clone();
        let highest_id = response
            .iter()
            .min_by(|lpost, rpost| lpost.id.cmp(&rpost.id))
            .unwrap()
            .id
            .clone();

        let next_request = match search {
            Search::Under(_) => {
                let lowest_id = response
                    .iter()
                    .min_by(|lpost, rpost| lpost.id.cmp(&rpost.id))
                    .unwrap()
                    .id
                    .clone();
                Search::Under(lowest_id)
            },

            Search::Above(_) => {
                let highest_id = response
                    .iter()
                    .max_by(|lpost, rpost| lpost.id.cmp(&rpost.id))
                    .unwrap()
                    .id
                    .clone();
                Search::Above(highest_id)
            },

            _ => unimplemented!(),
        };

        self.process_post_list(response.into_iter(), ctx);

        // notify self to perform the next page
        // NOTE: not like this tho; rethink the process
        ctx.notify(next_request);
    }
}

impl Actor for Danbooru {
    fn on_start(
        &mut self,
        ctx: &ContextImmutHalf<Self>,
    ) {
    }
}

impl Handles<Search> for Danbooru {
    type Response = ();

    fn handle(
        &mut self,
        msg: Search,
        ctx: &ContextImmutHalf<Self>,
    ) -> Self::Response {
        self.request(msg, ctx);
    }
}
