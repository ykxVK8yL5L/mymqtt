use std::process::CommandEnvs;
use std::{
    env,
    process,
    thread,
    time::Duration,
    fs,
    path::Path,
    collections::HashMap,
};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{de, Deserialize, Deserializer,Serialize};
use tracing::{info,debug,error,Level};
use tracing_subscriber::fmt::format;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use clap::{Parser, Arg};
use run_script;
use run_script::ScriptOptions;
extern crate paho_mqtt as mqtt;






pub type Cmds = Vec<Cmd>;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cmd {
    pub name: String,
    pub cmd: String,
}



#[derive(Debug, Deserialize,Serialize)]
struct MessageObj {
    msg:String,
    cmd: String,
}




/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "tcp://broker.emqx.io:1883")]
    host: String,
    #[clap(long, help="必需唯一ID", default_value = "rust_subscribe_sub")]
    client: String,
    #[clap(long, default_value = "rust/mqtt")]
    topic: String,
    #[clap(long, help="执行命令的键值映射文件，绝对路径", default_value = "~/cmd.json")]
    cmd: String,
}

const DFLT_QOS:i32=2;

// Reconnect to the broker when connection is lost.
fn try_reconnect(cli: &mqtt::Client) -> bool
{
    println!("Connection lost. Waiting to retry connection");
    for _ in 0..12 {
        thread::sleep(Duration::from_millis(5000));
        if cli.reconnect().is_ok() {
            println!("Successfully reconnected");
            return true;
        }
    }
    println!("Unable to reconnect after several attempts.");
    false
}

// Subscribes to multiple topics.
fn subscribe_topics(cli: &mqtt::Client,topic:&str) {
    if let Err(e) = cli.subscribe(topic,DFLT_QOS) {
        println!("Error subscribes topics: {:?}", e);
        process::exit(1);
    }

    // if let Err(e) = cli.subscribe_many(topic, DFLT_QOS) {
    //     println!("Error subscribes topics: {:?}", e);
    //     process::exit(1);
    // }
}

fn main() {
    let options = ScriptOptions::new();
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
    let cmd_path = Path::new(&args.cmd);
    let cmd_map_file = fs::read_to_string(cmd_path);
    let mut cmds:Cmds=vec![];
    match cmd_map_file {
        Ok(contet)=>{
            cmds = match serde_json::from_str(&contet){
                Ok(cmds)=>{cmds},
                Err(e)=>{
                    vec![]
                }
            }
        },
        Err(e)=>{
            error!("命令文件解析失败，可能无法执行命令.{}",e)
        }
    };

    let mut cmds_map = HashMap::new();
    for cmd in cmds {
        cmds_map.insert(cmd.name, cmd.cmd);
    }

    
    // Define the set of options for the create.
    // Use an ID for a persistent session.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(args.host)
        .client_id(args.client+&nowtime.to_string())
        .finalize();

    // Create a client.
    let mut cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    // Initialize the consumer before connecting.
    let rx = cli.start_consuming();

    // Define the set of options for the connection.
    let lwt = mqtt::MessageBuilder::new()
        .topic("test")
        .payload("Consumer lost connection")
        .finalize();
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    // Connect and wait for it to complete or fail.
    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    // Subscribe topics.
    subscribe_topics(&cli,&args.topic);

    println!("Processing requests...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            let message: MessageObj = match serde_json::from_str(&msg.payload_str()){
                Ok(msg_obj)=>{
                    msg_obj
                },
                Err(e)=>{
                    println!("{}", msg);
                    MessageObj{
                        msg:"".to_string(),
                        cmd:"".to_string(),
                    }
                }
            };
            if !&message.msg.is_empty(){
                println!("{}",&message.msg)
            }

            if !&message.cmd.is_empty(){
                match cmds_map.get(&message.cmd) {
                    Some(value)=>{
                        println!("cmd is:{}",value);
                        match run_script::run_script!(&value,&options) {
                            Ok((code,output,error))=>{
                                println!("执行结果:{}",output)
                            },
                            Err(e)=>{
                                error!("错误:{}",e)
                            } 
                        }
                    },
                    None=>{
                        error!("没找到相关命令:{}",&message.cmd)
                    }
                } 
            }

        }
        else if !cli.is_connected() {
            if try_reconnect(&cli) {
                println!("Resubscribe topics...");
                subscribe_topics(&cli,&args.topic);
            } else {
                break;
            }
        }
    }

    // If still connected, then disconnect now.
    if cli.is_connected() {
        println!("Disconnecting");
        cli.unsubscribe(&args.topic).unwrap();
        cli.disconnect(None).unwrap();
    }
    println!("Exiting");
}
