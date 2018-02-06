extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate egg_mode;
extern crate tokio_core;
extern crate regex;
extern crate rand;
extern crate pnet;

use std::fs::File;
use std::io::prelude::*;
use egg_mode::tweet::DraftTweet;
use egg_mode::media::{UploadBuilder, media_types};
use tokio_core::reactor::Core;
use regex::Regex;
use rand::distributions::{IndependentSample,Range};
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherTypes,EthernetPacket};
use pnet::util::MacAddr;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct Configuration {
    own_mac_addr: String,
    button_mac_addr: String,
    consumer_key: String,
    consumer_secret: String,
    access_token_key: String,
    access_token_secret: String,
}

#[derive(Debug)]
enum ShindoiPostError {
    UploadError(egg_mode::media::UploadError),
    TweetError(egg_mode::error::Error)
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

fn tweet_a_shindoi(ref token: &egg_mode::Token) -> Result<egg_mode::Response<egg_mode::tweet::Tweet>,ShindoiPostError> {
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
    let media_handler = core.run(upload_builder.call(&token,&handle)).map_err(|e| ShindoiPostError::UploadError(e))?;

    let tweet_draft = DraftTweet::new("こころがしんどい").media_ids(&[media_handler.id]);

    core.run(tweet_draft.send(&token, &handle)).map_err(|e| ShindoiPostError::TweetError(e))
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

    let interfaces = datalink::interfaces();
    let own_mac_addr = MacAddr::from_str(configuration.own_mac_addr.as_str()).expect("cannot load own MAC address..");
    let button_mac_addr = MacAddr::from_str(configuration.button_mac_addr.as_str()).expect("cannot load button's MAC address..");
    let broadcast_mac_addr = MacAddr::new(255u8, 255u8, 255u8, 255u8, 255u8, 255u8);

    let interface = interfaces.iter()
        .filter(|ifd| {
            if let Some(addr) = ifd.mac {
                addr == own_mac_addr
            } else {
                false
            }
        })
        .next()
        .expect("there are no NIC which has specified MAC address..");

    let mut rx = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(_, rx)) => rx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("error occurred when creating the channel: {}", e),
    };

    println!("Waiting a button's packets...");
    loop {
        match rx.next() {
            Ok(packet) => {
                let packet = EthernetPacket::new(packet).unwrap();
                match (packet.get_ethertype(), packet.get_source(), packet.get_destination()) {
                    (EtherTypes::Arp, src, dst) if dst == broadcast_mac_addr => {
                        println!("A ARP packet detected: from {}", src);
                        if src == button_mac_addr {
                            match tweet_a_shindoi(&token) {
                                Ok(status) => println!("Tweet status: {:?}", status),
                                err => println!("something error occurred.., stacktrace: {:?}", err),
                            };
                        }
                    }
                    _ => continue,
                }
            }
            Err(e) => panic!("error occurred while reading: {}", e),
        }
    }

}
