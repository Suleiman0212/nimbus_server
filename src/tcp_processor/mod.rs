use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpStream,
    path::PathBuf,
};

use dtp::{Content, ContentType, Message, SubTitile, Title};
use rw::{send_message, send_ok};

// Get and guide requests from client
pub fn handle_connection(stream: &mut TcpStream, dir: &str) -> Result<(), Box<dyn Error>> {
    let msg: Message = rw::get_message(stream)?;
    match msg.title {
        Title::GetRequest => handle_get_request(stream, msg, dir)?,
        Title::SendRequest => handle_send_request(stream, msg, dir)?,
        Title::FileListRequest => handle_fl_request(stream, msg, dir)?,
        Title::RemoveRequest => handle_remove_request(stream, msg, dir)?,
    }
    Ok(())
}

// Handling "Donwload" request
// Steps:
// 1. Get file name and if file exist send file_size
// 2. Wait ok after sending file_size
// 3. Send file binary data
fn handle_get_request(
    stream: &mut TcpStream,
    msg: Message,
    dir: &str,
) -> Result<(), Box<dyn Error>> {
    let file_name = match unbox_message(msg, Title::GetRequest, ContentType::FileName)?[0].clone() {
        Content::Text(t) => t,
        _ => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "InvalidType: file name incorrect type (isnt Text).",
            )))
        }
    };
    let path = PathBuf::new().join(dir).join(file_name);

    match fs::is_file_exist(&path) {
        Ok(_) => (),
        Err(_) => rw::send_err(
            stream,
            Title::GetRequest,
            "GetRequest error: File didint exist!",
        )?,
    }

    let file_size = Content::Number(fs::file_size(&path)?);
    let answer: Message = Message::new(
        Title::GetRequest,
        SubTitile::Ok,
        ContentType::FileSize,
        vec![file_size],
    );

    rw::send_message(stream, answer)?;

    rw::wait_ok(stream, Title::GetRequest)?;

    let mut buf: Vec<u8> = vec![];
    let mut file = fs::load_file(&path)?;
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

// Handling "Upload" request
// Steps:
// 1. Get file name
// 2. Send OK
// 3. Get file binary data
// 4. Send OK
fn handle_send_request(
    stream: &mut TcpStream,
    msg: Message,
    dir: &str,
) -> Result<(), Box<dyn Error>> {
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
    let path = PathBuf::new().join(dir).join(file_name);

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

    let mut file = fs::create_file(&path)?;
    file.write_all(&file_data)?;

    send_ok(stream, Title::SendRequest)?;

    Ok(())
}

// Handling "Files-List" request
// Steps:
// 1. Get files in server directory
// 2. Send files list
fn handle_fl_request(
    stream: &mut TcpStream,
    msg: Message,
    dir: &str,
) -> Result<(), Box<dyn Error>> {
    unbox_message(msg, Title::FileListRequest, ContentType::NoContent)?;

    let path = PathBuf::new().join(dir);
    let files = fs::files_list(&path)?;
    let files = Content::Binary(files.as_bytes().to_vec());
    let message = Message::new(
        Title::FileListRequest,
        SubTitile::Ok,
        ContentType::FileData,
        vec![files],
    );

    rw::send_message(stream, message)?;
    Ok(())
}

// Handling "Delete" request
// Steps:
// 1. Get file name
// 2. Delete file
// 3. Send OK
fn handle_remove_request(
    stream: &mut TcpStream,
    msg: Message,
    dir: &str,
) -> Result<(), Box<dyn Error>> {
    let file_name =
        match unbox_message(msg, Title::RemoveRequest, ContentType::FileName)?[0].clone() {
            Content::Text(t) => t,
            _ => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "InvalidType: file name incorrect type (isnt Text).",
                )))
            }
        };

    let path = PathBuf::new().join(dir).join(file_name);
    match fs::remove(&path) {
        Ok(_) => (),
        Err(err) => {
            let err_str = err.to_string();
            rw::send_err(stream, Title::RemoveRequest, &err_str)?;
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("RemoveDir error: {err}"),
            )));
        }
    }

    send_ok(stream, Title::RemoveRequest)?;

    Ok(())
}

// Unboxing message like a gift
// It needed to catch any errors
// Related with incorrect types
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
