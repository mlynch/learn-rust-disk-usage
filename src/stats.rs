use std::{borrow::Borrow, path::PathBuf};

use colored::*;
use glob::Pattern;

use crate::utils::bytes_to_human;

type LargeFile = (String, u64);

pub struct AnalyzerStats {
    developer_dir_pattern: Pattern,
    pub largest_files: Box<Vec<LargeFile>>,
    pub developer_dirs: Vec<LargeFile>,
    pub total_music: u64,
    pub total_images: u64,
    pub total_videos: u64,
    pub total_documents: u64,
    pub total_binaries: u64,
    pub total_archives: u64,
    pub total_other: u64,
    pub dev_total_usage: u64
}

impl AnalyzerStats {
    pub fn new() -> AnalyzerStats {
        AnalyzerStats {
            //largest_files: PriorityQueue::with_capacity(100),
            developer_dir_pattern: Pattern::new("**/node_modules").unwrap(),
            largest_files: Box::new(vec![]),
            developer_dirs: vec![],
            total_music: 0,
            total_images: 0,
            total_videos: 0,
            total_documents: 0,
            total_binaries: 0,
            total_archives: 0,
            total_other: 0,
            dev_total_usage: 0
        }
    }

    pub fn get_largest(&self) -> &Vec<LargeFile> {
        return self.largest_files.borrow();
    }

    pub fn register_dir_usage(&mut self, path: &PathBuf, len: u64) {
        let dirname = path.file_name().expect("Unable to process dir").to_str().unwrap();

        if self.developer_dir_pattern.matches(dirname) {
            self.dev_total_usage += len;
            self.developer_dirs.push((String::from(path.to_str().unwrap()), len));
        }
    }

    pub fn register_file(&mut self, path_str: &str, len: u64, nlargest: u64, large_bytes: u64) {
        // println!("{}", path_str);

        let mut mime_str = String::from("");
        if let Some(mime) = mime_guess::from_path(path_str).first() {
            mime_str = mime.to_string();
        };

        self.push_largest(path_str, len, nlargest, large_bytes);


        if mime_str.contains("image/") {
            self.total_images += len;
        } else if mime_str.contains("audio/") {
            self.total_music += len;
        } else if mime_str.contains("video/") {
            self.total_videos += len;
        } else if self.is_document(&mime_str) {
            self.total_documents += len;
        } else if self.is_archive(&mime_str) {
            self.total_archives += len;
        } else if self.is_binary(&mime_str) {
            self.total_binaries += len;
        } else {
            self.total_other += len;
        }
    }

    fn is_document(&self, mime: &String) -> bool {
        match mime.as_str() {
            "application/msword" => true,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => true,
            "application/vnd.ms-excel" => true,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => true,
            "text/plain" => true,
            _ => false
        }
    }

    fn is_binary(&self, mime: &String) -> bool {
        match mime.as_str() {
            "application/octet-stream" => true,
            _ => false
        }
    }

    fn is_archive(&self, mime: &String) -> bool {
        match mime.as_str() {
            "application/x-bzip" => true,
            "application/x-bzip2" => true,
            "application/zip" => true,
            "application/x-tar" => true,
            "application/gzip" => true,
            "application/x-7z-compressed" => true,
            _ => false
        }
    }

    pub fn push_largest(&mut self, path_str: &str, len: u64, nlargest: u64, large_bytes: u64) {
        if len < large_bytes as u64 {
            return;
        }

        if self.largest_files.len() == 0 {
            self.largest_files.push((path_str.to_string(), len));
            self.largest_files.sort_by(|a, b| b.1.cmp(&a.1));
        } else { //else if self.largest_files.iter().any(|x| len > x.1) {
            self.largest_files.push((path_str.to_string(), len));
            self.largest_files.sort_by(|a, b| b.1.cmp(&a.1));
        }

        if self.largest_files.len() > nlargest as usize {
            self.largest_files.truncate(nlargest as usize);
        }
    }

    pub fn print_largest(&self) {
        let largest: &Box<Vec<(String, u64)>> = self.largest_files.borrow();

        for s in largest.iter() {
            println!("{} ({})", s.0.bright_white(), bytes_to_human(s.1).bold());
        }
    }
}
