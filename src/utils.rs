use std::path::PathBuf;

use human_bytes::human_bytes;

pub fn is_hidden(path: &PathBuf) -> bool {
    let path_str = path.file_name().unwrap().to_str().unwrap();
    path_str.starts_with(".")
}

pub fn bytes_to_human(len: u64) -> String {
    return human_bytes(len as f64);
}