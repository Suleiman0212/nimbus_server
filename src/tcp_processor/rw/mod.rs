use super::dtp::{ContentType, Message, SubTitile, Title};
use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpStream,
};

fn read(stream: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut temp_buf = [0; 1024];
    let mut data = Vec::new();

    loop {
        let bytes_read = stream.read(&mut temp_buf)?;
        if bytes_read == 0 {
            break;
        }
        data.extend_from_slice(&temp_buf[..bytes_read]);

        // Using marker to find message end
        if data.ends_with(b"END") {
            data.truncate(data.len() - 3); // Deleting the marker
            break;
        }
    }

    // If coonection was lost or break,
    // we catch it ends_with using len()
    if data.len() == 0 {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Received data = 0, maybe connection was break.",
        )));
    }
    Ok(data)
}

fn write(stream: &mut TcpStream, bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut message = bytes;
    message.extend_from_slice(b"END"); // Addint marker to the end
    stream.write_all(&message)?;
    stream.flush()?;
    Ok(())
}

pub fn get_message(stream: &mut TcpStream) -> Result<Message, Box<dyn Error>> {
    let data: Vec<u8> = read(stream)?;
    Ok(Message::from_bytes(data)?)
}

pub fn send_message(stream: &mut TcpStream, message: Message) -> Result<(), Box<dyn Error>> {
    let bytes = message.as_bytes()?;
    write(stream, bytes)?;
    Ok(())
}

// Function needed to answer to message without any data
pub fn send_ok(stream: &mut TcpStream, ok_title: Title) -> Result<(), Box<dyn Error>> {
    let ok_msg: Message = Message::new(ok_title, SubTitile::Ok, ContentType::NoContent, vec![]);
    send_message(stream, ok_msg)?;
    Ok(())
}

// Function used for wait OK message and send another data
pub fn wait_ok(stream: &mut TcpStream, ok_title: Title) -> Result<(), Box<dyn Error>> {
    let msg = get_message(stream)?;
    if msg.title != ok_title {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "WaitOk: title content incorrect type.",
        )));
    }
    match msg.sub_title {
        SubTitile::Ok => (),
        _ => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "WaitOk: sub_title content incorrect type.",
            )))
        }
    }
    Ok(())
}
