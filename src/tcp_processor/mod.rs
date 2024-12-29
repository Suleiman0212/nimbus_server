use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpStream,
};

use dtp::{Content, ContentType, Message, SubTitile, Title};
use rw::{send_message, send_ok};

mod dtp;
mod fs;
mod rw;

const FILE_DIR: &str = "/home/zeroone/server_data/";

pub fn handle_connection(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let msg: Message = rw::get_message(stream)?;
    match msg.title {
        Title::GetRequest => handle_get_request(stream, msg)?,
        Title::SendRequest => handle_send_request(stream, msg)?,
    }
    Ok(())
}

fn handle_get_request(stream: &mut TcpStream, msg: Message) -> Result<(), Box<dyn Error>> {
    let file_name = match unbox_message(msg, Title::GetRequest, ContentType::FileName)?[0].clone() {
        Content::Text(t) => t,
        _ => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "InvalidType: file name incorrect type (isnt Text).",
            )))
        }
    };

    match fs::is_file_exist(FILE_DIR, &file_name) {
        Ok(_) => (),
        Err(_) => {
            send_no_exist_error(stream, Title::GetRequest)?;
            // TODO: add return err
        }
    }

    let file_size = Content::Number(fs::file_size(FILE_DIR, &file_name)?);
    let answer: Message = Message::new(
        Title::GetRequest,
        SubTitile::Ok,
        ContentType::FileSize,
        vec![file_size],
    );

    rw::send_message(stream, answer)?;

    rw::wait_ok(stream, Title::GetRequest)?;

    let mut buf: Vec<u8> = vec![];
    let mut file = fs::load_file(FILE_DIR, &file_name)?;
    file.read_to_end(&mut buf)?;
    let file_data = Content::Binary(buf);

    let file_message: Message = Message::new(
        Title::GetRequest,
        SubTitile::Ok,
        ContentType::FileData,
        vec![file_data],
    );

    send_message(stream, file_message)?;

    Ok(())
}

fn handle_send_request(stream: &mut TcpStream, msg: Message) -> Result<(), Box<dyn Error>> {
    let file_name = match unbox_message(msg, Title::SendRequest, ContentType::FileName)?[0].clone()
    {
        Content::Text(t) => t,
        _ => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "InvalidType: file name incorrect type (isnt Text).",
            )))
        }
    };

    rw::send_ok(stream, Title::SendRequest)?;

    let file_message = rw::get_message(stream)?;
    let file_data =
        match unbox_message(file_message, Title::SendRequest, ContentType::FileData)?[0].clone() {
            Content::Binary(b) => b,
            _ => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "InvalidType: file content incorrect type (isnt Binary).",
                )))
            }
        };

    let mut file = fs::create_file(FILE_DIR, &file_name)?;
    file.write_all(&file_data)?;

    send_ok(stream, Title::SendRequest)?;

    Ok(())
}

fn send_no_exist_error(stream: &mut TcpStream, title: Title) -> Result<(), Box<dyn Error>> {
    let error_message: Message = Message::new(
        title,
        SubTitile::Err,
        ContentType::ErrMessage,
        vec![Content::Text("File didnt exist!".to_string())],
    );
    send_message(stream, error_message)?;
    Ok(())
}

fn unbox_message(
    message: Message,
    ok_title: Title,
    ok_content_type: ContentType,
) -> Result<Vec<Content>, Box<dyn Error>> {
    if message.title != ok_title {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "IncorrectMessage: title content incorrect type.",
        )));
    }

    match message.sub_title {
        SubTitile::Ok => (),
        _ => {
            let e = match message.content_array[0].clone() {
                Content::Text(t) => t,
                _ => "<Cant read error message, incorrect type.>".to_string(),
            };
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("ErrorMessage: {e}"),
            )));
        }
    }

    if message.content_type != ok_content_type {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "IncorrectMessage: content type is incorrect.",
        )));
    }

    Ok(message.content_array)
}
