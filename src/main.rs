extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate egg_mode;
extern crate tokio_core;

use std::fs::File;
use egg_mode::tweet::DraftTweet;
use tokio_core::reactor::Core;

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

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let post = core.run(DraftTweet::new("Tweet from Rust").send(&token, &handle)).unwrap();

    println!("{:?}", post);
}
