#![allow(unused)]
use std::{
    error::Error,
    fs::{self, DirBuilder, File},
    io::{self, Read},
    path::Path,
};

pub fn is_file_exist(path: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let file_path_str: &str = &format!("{path}{name}");
    let file_path = Path::new(&file_path_str);
    if !file_path.exists() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "FilePath: didnt exist.",
        )));
    }
    Ok(())
}

pub fn load_file(path: &str, name: &str) -> Result<File, Box<dyn Error>> {
    let file_path_str: &str = &format!("{path}{name}");
    let file_path = Path::new(file_path_str);

    is_file_exist(path, name);

    let file: File = File::open(format!("{path}{name}"))?;
    Ok(file)
}

pub fn create_file(path: &str, name: &str) -> Result<File, Box<dyn Error>> {
    let dir_path = Path::new(&path);

    is_file_exist(path, name);

    let file: File = File::create(format!("{path}{name}"))?;
    Ok(file)
}

pub fn file_size(path: &str, name: &str) -> Result<u64, Box<dyn Error>> {
    let file_metadata = fs::metadata(format!("{path}{name}"))?;
    Ok(file_metadata.len())
}
