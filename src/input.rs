use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use crate::utils::{file_exists, get_file_size, human_readable_size};

pub enum InputSource {
    Stdin,
    File(String),
    Directory(String),
}

pub struct InputConfig {
    pub buffer_size: usize,
    pub follow_mode: bool,
    pub tail_lines: Option<usize>,
    pub max_file_size: Option<u64>,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192, // 8KB buffer for efficient reading
            follow_mode: false,
            tail_lines: None,
            max_file_size: Some(1024 * 1024 * 1024), // 1GB max by default
        }
    }
}

impl InputSource {
    pub fn reader(&self, config: &InputConfig) -> io::Result<Box<dyn BufRead>> {
        match self {
            InputSource::Stdin => Ok(Box::new(BufReader::with_capacity(config.buffer_size, io::stdin()))),
            InputSource::File(path) => {
                let file_size = get_file_size(path).unwrap_or(0);
                
                // Check file size limits
                if let Some(max_size) = config.max_file_size {
                    if file_size > max_size {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("File too large: {} (max: {})", 
                                human_readable_size(file_size), 
                                human_readable_size(max_size))
                        ));
                    }
                }
                
                let file = File::open(path)?;
                let mut reader = BufReader::with_capacity(config.buffer_size, file);
                
                // Handle tail mode
                if let Some(tail_lines) = config.tail_lines {
                    if tail_lines > 0 {
                        Self::seek_to_tail(&mut reader, tail_lines, file_size)?;
                    }
                }
                
                Ok(Box::new(reader))
            }
            InputSource::Directory(_path) => {
                Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "Directory input not yet implemented"
                ))
            }
        }
    }
    
    pub fn get_info(&self) -> Option<String> {
        match self {
            InputSource::Stdin => Some("stdin".to_string()),
            InputSource::File(path) => {
                if file_exists(path) {
                    if let Some(size) = get_file_size(path) {
                        Some(format!("{} ({})", path, human_readable_size(size)))
                    } else {
                        Some(path.clone())
                    }
                } else {
                    Some(format!("{} (not found)", path))
                }
            }
            InputSource::Directory(path) => Some(format!("directory: {}", path)),
        }
    }
    
    fn seek_to_tail<R: BufRead + Seek>(reader: &mut R, tail_lines: usize, file_size: u64) -> io::Result<()> {
        if file_size == 0 {
            return Ok(());
        }
        
        // Start from end and work backwards
        let mut pos = file_size;
        let mut lines_found = 0;
        let mut buffer = vec![0; 4096];
        
        while pos > 0 && lines_found <= tail_lines {
            let chunk_size = std::cmp::min(4096, pos as usize);
            pos -= chunk_size as u64;
            
            reader.seek(SeekFrom::Start(pos))?;
            reader.read_exact(&mut buffer[..chunk_size])?;
            
            // Count newlines in this chunk
            for byte in buffer[..chunk_size].iter().rev() {
                if *byte == b'\n' {
                    lines_found += 1;
                    if lines_found > tail_lines {
                        break;
                    }
                }
            }
        }
        
        // Seek to the position where we found the right number of lines
        reader.seek(SeekFrom::Start(pos))?;
        
        Ok(())
    }
}

pub fn resolve_input(input_arg: &Option<String>) -> InputSource {
    match input_arg {
        Some(path) => {
            if Path::new(path).is_dir() {
                InputSource::Directory(path.clone())
            } else {
                InputSource::File(path.clone())
            }
        }
        None => InputSource::Stdin,
    }
}

pub struct LogStream {
    reader: Box<dyn BufRead>,
    config: InputConfig,
    line_buffer: String,
}

impl LogStream {
    pub fn new(source: &InputSource, config: InputConfig) -> io::Result<Self> {
        let reader = source.reader(&config)?;
        Ok(Self {
            reader,
            config,
            line_buffer: String::new(),
        })
    }
    
    pub fn next_line(&mut self) -> io::Result<Option<String>> {
        self.line_buffer.clear();
        
        match self.reader.read_line(&mut self.line_buffer)? {
            0 => Ok(None), // EOF
            _ => {
                // Remove trailing newline and return
                if self.line_buffer.ends_with('\n') {
                    self.line_buffer.pop();
                    if self.line_buffer.ends_with('\r') {
                        self.line_buffer.pop();
                    }
                }
                Ok(Some(self.line_buffer.clone()))
            }
        }
    }
    
    pub fn get_config(&self) -> &InputConfig {
        &self.config
    }
}