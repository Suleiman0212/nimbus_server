use config::Config;
use fs;
use std::{
    error::Error,
    net::{SocketAddr, TcpListener},
};

mod config;
mod tcp_processor;

fn main() -> Result<(), Box<dyn Error>> {
    // Check config file for existance
    // If it isnt, create it
    let conf: Config = match fs::is_file_exist(&config::get_config_path()?.join("config.toml")) {
        Ok(_) => config::get_config()?,
        Err(e) => {
            eprintln!("{e}");
            config::crete_conf()?;
            println!(
                "Config created in {}",
                &config::get_config_path()?.to_str().unwrap()
            );
            println!("Check and edit config data_path.");
            config::get_config()?
        }
    };

    // Bind listener
    let addr = conf.ip.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(addr).expect("Can't run server:");

    // Handling incoming streams
    for stream in listener.incoming() {
        let mut tcp_stream = stream?;
        match tcp_processor::handle_connection(&mut tcp_stream, &conf.data_dir) {
            Ok(_) => println!("Connection was handled without an error."),
            Err(e) => eprintln!("{e}"),
        };
    }
    Ok(())
}
