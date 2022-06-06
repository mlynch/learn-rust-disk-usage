use priority_queue::PriorityQueue;

use crate::utils::bytes_to_human;

pub struct AnalyzerStats {
    largest_files: PriorityQueue<String, u64>
}

impl AnalyzerStats {
    pub fn new() -> AnalyzerStats {
        AnalyzerStats {
            largest_files: PriorityQueue::with_capacity(100)
        }
    }
    pub fn register_file(&mut self, path_str: &str, len: u64) {
        self.largest_files.push(path_str.to_owned(), len);
    }

    pub fn print_largest(&self) {
        println!("Largest: {}", self.largest_files.len());

        let largest_files = self.largest_files.clone();

        let sorted = largest_files.into_sorted_iter();

        for (path, len) in sorted.take(100) {
            println!("{} ({})", path, bytes_to_human(len));
        }
    }
}
