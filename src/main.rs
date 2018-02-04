extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate egg_mode;
extern crate tokio_core;
extern crate regex;
extern crate rand;

use std::fs::File;
use std::io::prelude::*;
use egg_mode::tweet::DraftTweet;
use egg_mode::media::{UploadBuilder, media_types};
use tokio_core::reactor::Core;
use regex::Regex;
use rand::distributions::{IndependentSample,Range};

#[derive(Debug, Deserialize)]
struct Configuration {
    mac_addr: String,
    consumer_key: String,
    consumer_secret: String,
    access_token_key: String,
    access_token_secret: String,
}

impl Configuration {
    fn new(file_name: &str) -> Self {
        let file = File::open(file_name).expect("error occured when loading configuration...");
        serde_yaml::from_reader(file).expect("error occuered when parsing configuration yaml...")
    }
}

fn select_a_image() -> String {
    let pathes = std::fs::read_dir("img/").expect("there is really existing img directory?");

    let pattern = Regex::new(r".+\.(JPG|jpg)$").unwrap();

    let jpg_files: Vec<String> = pathes.map(|p| p.ok().map(|q| q.file_name().into_string().ok()).and_then(|q| q))
        .filter(|p| p.clone().map(|q| pattern.is_match(&q)).unwrap_or(false)).map(|p| p.unwrap()).collect();

    let mut rng = rand::thread_rng();
    let range = Range::new(0,jpg_files.len());

    let index = range.ind_sample(&mut rng);

    format!("img/{}",jpg_files[index].clone())
}

fn main() {
    let configuration = Configuration::new("configuration.yml");

    let consumer_token = egg_mode::KeyPair::new(configuration.consumer_key,
                                                configuration.consumer_secret);
    let access_token = egg_mode::KeyPair::new(configuration.access_token_key,
                                              configuration.access_token_secret);

    let token = egg_mode::Token::Access {
        consumer: consumer_token,
        access: access_token,
    };

    let shindoi_picture = select_a_image();
    let mut buffer = Vec::new();

    {
        let mut file = File::open(shindoi_picture.clone()).expect("cannot open picture file..");
        let _ = file.read_to_end(&mut buffer).expect("cannot read picture file..");
    }

    println!("{}",shindoi_picture);

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let upload_builder = UploadBuilder::new(buffer, media_types::image_jpg());
    let media_handler = core.run(upload_builder.call(&token,&handle)).expect("handling media failed..");

    let tweet_draft = DraftTweet::new("Tweet from Rust").media_ids(&[media_handler.id]);

    let post = core.run(tweet_draft.send(&token, &handle)).expect("tweet failed..");

    println!("{:?}", post);
}
