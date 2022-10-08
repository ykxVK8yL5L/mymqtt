use std::{
    env,
    process,
    time::Duration
};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{de, Deserialize, Deserializer};
use tracing::{info,debug,error,Level};
use tracing_subscriber::fmt::format;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use clap::{Parser, Arg};
extern crate paho_mqtt as mqtt;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "tcp://broker.emqx.io:1883")]
    host: String,
    #[clap(long,help="必需唯一ID", default_value = "rust_publish_pub")]
    client: String,
    #[clap(long, default_value = "rust/mqtt")]
    topic: String,
    #[clap(long, default_value = "hello world")]
    msg: String,
}
// Define the qos.
const QOS:i32 = 2;
fn main() {

    tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_env("RUST_MQTT_LOG"))
    .init();
    if env::var("RUST_MQTT_LOG").is_err() {
        env::set_var("RUST_MQTT_LOG", "rust_mqtt=info");
    }
    let nowtime = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis();
    let args = Args::parse();
    // Define the set of options for the create.
    // Use an ID for a persistent session.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(args.host)
        .client_id(args.client+&nowtime.to_string())
        .finalize();

    // Create a client.
    let cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    // Define the set of options for the connection.
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail.
    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    // Create a message and publish it.
    // Publish message to 'test' and 'hello' topics.
   
    let msg = mqtt::Message::new(&args.topic, args.msg, QOS);
    let tok = cli.publish(msg);
    if let Err(e) = tok {
            println!("Error sending message: {:?}", e);
    }
    
    // Disconnect from the broker.
    let tok = cli.disconnect(None);
    println!("Disconnect from the broker");
    tok.unwrap();
}
