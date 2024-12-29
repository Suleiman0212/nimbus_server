use std::{
    error::Error,
    net::{SocketAddr, TcpListener},
};

mod tcp_processor;

fn main() -> Result<(), Box<dyn Error>> {
    // Bind listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).expect("Can't run server:");

    // Handling incoming streams
    for stream in listener.incoming() {
        let mut tcp_stream = stream?;
        match tcp_processor::handle_connection(&mut tcp_stream) {
            Ok(_) => println!("Connection was handled without errors."),
            Err(e) => eprintln!("{e}"),
        };
    }
    Ok(())
}
