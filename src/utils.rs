use chrono::{DateTime, Utc};
use std::path::Path;

pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn get_file_size(path: &str) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

pub fn human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let bytes = bytes as f64;
    let base = 1024_f64;
    let i = (bytes.ln() / base.ln()).floor() as usize;
    let size = bytes / base.powi(i as i32);
    
    if i < UNITS.len() {
        format!("{:.1} {}", size, UNITS[i])
    } else {
        format!("{} B", bytes as u64)
    }
}