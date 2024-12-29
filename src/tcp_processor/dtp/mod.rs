//DTP - Data Transfer Protocol also NimbusDTP.
//Suleiman Kirimov
//2024-12-16
//v0.1

#![allow(unused)]
#[derive(Debug)]
pub struct Message {
    pub title: Title,
    pub sub_title: SubTitile,
    pub content_type: ContentType,
    pub content_array: Vec<Content>,
}

impl Message {
    pub fn new(
        title: Title,
        sub_title: SubTitile,
        content_type: ContentType,
        content_array: Vec<Content>,
    ) -> Self {
        Self {
            title,
            sub_title,
            content_type,
            content_array,
        }
    }
    pub fn as_bytes(&self) -> Result<Vec<u8>, &'static str> {
        let mut bytes: Vec<u8> = vec![];
        bytes.push(self.title.value());
        bytes.push(self.sub_title.value());
        bytes.push(self.content_type.value());
        for content in &self.content_array {
            match content {
                Content::Text(t) => {
                    if t.len() <= 255 {
                        let mut text_bytes = t.as_bytes().to_vec();
                        text_bytes.resize(255, 0);
                        bytes.extend(text_bytes);
                    } else {
                        return Err("String len more than 255!");
                    }
                }
                Content::Number(n) => {
                    bytes.extend(n.to_le_bytes().to_vec());
                }
                Content::Binary(b) => {
                    bytes.extend(b.to_vec());
                }
            }
        }
        Ok(bytes)
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Message, &'static str> {
        let title = match bytes[0] {
            Title::GET_REQUEST => Title::GetRequest,
            Title::SEND_REQUEST => Title::SendRequest,
            _ => return Err("Unknown title type!"),
        };
        let sub_title = match bytes[1] {
            SubTitile::OK => SubTitile::Ok,
            SubTitile::ERR => SubTitile::Err,
            _ => return Err("Unknown sub_title type!"),
        };
        let content_type = match bytes[2] {
            ContentType::NO_CONTENT => ContentType::NoContent,
            ContentType::ERR_MESSAGE => ContentType::ErrMessage,
            ContentType::FILE_NAME => ContentType::FileName,
            ContentType::FILE_SIZE => ContentType::FileSize,
            ContentType::FILE_DATA => ContentType::FileData,
            _ => return Err("Unknown content_type type!"),
        };
        let content_array: Vec<Content> = match content_type {
            ContentType::NoContent => vec![],
            ContentType::ErrMessage => {
                vec![Content::text_from_range_bytes(bytes[3..].to_vec())]
            }
            ContentType::FileName => {
                vec![Content::text_from_range_bytes(bytes[3..].to_vec())]
            }
            ContentType::FileSize => vec![Content::number_from_bytes(
                bytes[3..11]
                    .try_into()
                    .expect("Cant convert [u8] to [u8; 8]"),
            )],
            ContentType::FileData => vec![Content::Binary(bytes[3..].to_vec())],
            _ => return Err("Unknown content_type type!"),
        };
        let message: Message = Message::new(title, sub_title, content_type, content_array);
        Ok(message)
    }
}

#[derive(Debug, PartialEq)]
pub enum Title {
    GetRequest,
    SendRequest,
}

impl Title {
    const GET_REQUEST: u8 = 0;
    const SEND_REQUEST: u8 = 1;

    pub fn value(&self) -> u8 {
        match self {
            Title::GetRequest => Self::GET_REQUEST,
            Title::SendRequest => Self::SEND_REQUEST,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SubTitile {
    Ok,
    Err,
}

impl SubTitile {
    const OK: u8 = 0;
    const ERR: u8 = 1;

    pub fn value(&self) -> u8 {
        match self {
            SubTitile::Ok => Self::OK,
            SubTitile::Err => Self::ERR,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ContentType {
    NoContent,
    ErrMessage,
    FileName,
    FileSize,
    FileData,
}

impl ContentType {
    const NO_CONTENT: u8 = 0;
    const ERR_MESSAGE: u8 = 1;
    const FILE_NAME: u8 = 2;
    const FILE_SIZE: u8 = 3;
    const FILE_DATA: u8 = 4;

    pub fn value(&self) -> u8 {
        match self {
            ContentType::NoContent => Self::NO_CONTENT,
            ContentType::ErrMessage => Self::ERR_MESSAGE,
            ContentType::FileName => Self::FILE_NAME,
            ContentType::FileSize => Self::FILE_SIZE,
            ContentType::FileData => Self::FILE_DATA,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Content {
    Text(String),
    Number(u64),
    Binary(Vec<u8>),
}

impl Content {
    fn text_from_range_bytes(bytes: Vec<u8>) -> Content {
        let trimmed_bytes: Vec<u8> = bytes.into_iter().take_while(|&x| x != 0).collect();
        let text: String = String::from_utf8_lossy(&trimmed_bytes).to_string();
        Content::Text(text)
    }
    fn number_from_bytes(bytes: [u8; 8]) -> Content {
        let number: u64 = u64::from_le_bytes(bytes);
        Content::Number(number)
    }
}
