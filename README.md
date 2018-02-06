# KokoroGaShindoi Dash Button
When you pushed Amazon Dash Button, this service can tweet "KokoroGaShindoi" and a picture which express it.

# How to use 
 1. Install cargo (rustc).
 1. Execute "cargo build".
 1. Set some picture in img/.
 1. Edit "configuraiton.yml" as follows. 
 ```
own_mac_addr: "your NIC MAC address"
button_mac_addr: "your dash button's MAC address"
consumer_key: "twitter consumer key"
consumer_secret: "twitter consumer key secret"
access_token_key: "twitter access token key"
access_token_secret: "twitter access token secret"
 ```
1. execute "cargo run" or binary.
