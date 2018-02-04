extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::fs::File;

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
    println!("Hello, world!");
}
