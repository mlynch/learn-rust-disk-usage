use std::path::PathBuf;

pub fn is_hidden(path: &PathBuf) -> bool {
    let path_str = path.file_name().unwrap().to_str().unwrap();
    path_str.starts_with(".")
}