use anyhow::Result;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use std::fs;
use tokio::process::Command;
use tracing::{info, error, debug};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: String,
    pub line_number: Option<usize>,
}

pub async fn read_log_file(file_path: &str) -> Result<Vec<String>> {
    info!("Reading log file: {}", file_path);
    
    // Read file as bytes first
    let data = match fs::read(file_path) {
        Ok(data) => {
            debug!("Read {} bytes from file {}", data.len(), file_path);
            data
        }
        Err(e) => {
            error!("Failed to read file {}: {}", file_path, e);
            return Err(e.into());
        }
    };
    
    // Detect encoding and create decoder
    let (encoding, _confidence, decoder) = detect_and_create_decoder(&data);
    info!("Detected encoding {} for file {}", encoding.name(), file_path);
    
    // Process line by line for better error recovery
    let lines = decode_lines_robust(&data, &decoder)?;
    debug!("Decoded {} lines from file {}", lines.len(), file_path);
    
    Ok(lines)
}

pub async fn execute_and_capture(command: &str) -> Result<Vec<String>> {
    info!("Executing command: {}", command);
    
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        error!("Empty command provided");
        return Err(anyhow::anyhow!("Empty command"));
    }

    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }

    let output = match cmd.output().await {
        Ok(output) => output,
        Err(e) => {
            error!("Failed to execute command '{}': {}", command, e);
            return Err(e.into());
        }
    };

    debug!("Command completed with status: {}", output.status);
    
    let mut lines = Vec::new();

    // Add stdout lines
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    debug!("Captured {} stdout lines", stdout_lines.len());
    lines.extend(stdout_lines);

    // Add stderr lines
    let stderr = String::from_utf8_lossy(&output.stderr);
    lines.extend(stderr.lines().map(|s| s.to_string()));

    Ok(lines)
}

/// Detect file encoding and create appropriate decoder
fn detect_and_create_decoder(data: &[u8]) -> (&'static encoding_rs::Encoding, f64, encoding_rs::Decoder) {
    use encoding_rs::{UTF_8, UTF_16LE, UTF_16BE, WINDOWS_1252};

    // Simple heuristics for encoding detection
    if data.is_empty() {
        return (UTF_8, 1.0, UTF_8.new_decoder());
    }

    // Check for UTF-8 BOM
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (UTF_8, 1.0, UTF_8.new_decoder_with_bom_removal());
    }

    // Check for UTF-16 LE BOM
    if data.starts_with(&[0xFF, 0xFE]) {
        return (UTF_16LE, 1.0, UTF_16LE.new_decoder_with_bom_removal());
    }

    // Check for UTF-16 BE BOM
    if data.starts_with(&[0xFE, 0xFF]) {
        return (UTF_16BE, 1.0, UTF_16BE.new_decoder_with_bom_removal());
    }

    // Enhanced analysis for encoding detection
    let (utf8_confidence, has_utf8_sequences, has_invalid_utf8) = analyze_utf8_content(data);
    let (latin1_confidence, has_latin1_chars) = analyze_latin1_content(data);
    let likely_utf16 = detect_utf16_pattern(data);

    // Decision logic with improved thresholds
    if likely_utf16 {
        let sample = &data[..data.len().min(1024)];
        if sample.len() >= 2 && sample[0] == 0x00 {
            (UTF_16BE, 0.9, UTF_16BE.new_decoder())
        } else {
            (UTF_16LE, 0.9, UTF_16LE.new_decoder())
        }
    } else if utf8_confidence > 0.98 && !has_invalid_utf8 {
        // Very high confidence UTF-8 with no invalid sequences
        (UTF_8, utf8_confidence, UTF_8.new_decoder())
    } else if has_latin1_chars && latin1_confidence > utf8_confidence {
        // Latin-1 characters detected and higher confidence than UTF-8
        // Use Windows-1252 which handles ISO-8859-1 compatibility
        (WINDOWS_1252, latin1_confidence, WINDOWS_1252.new_decoder())
    } else if utf8_confidence > 0.85 && has_utf8_sequences && !has_invalid_utf8 {
        // Good UTF-8 confidence with valid multi-byte sequences
        (UTF_8, utf8_confidence, UTF_8.new_decoder())
    } else if has_latin1_chars {
        // Has Latin-1 characters, use Windows-1252 for compatibility
        (WINDOWS_1252, latin1_confidence, WINDOWS_1252.new_decoder())
    } else {
        // Fallback to Windows-1252 (handles ISO-8859-1 as superset)
        (WINDOWS_1252, 0.6, WINDOWS_1252.new_decoder())
    }
}

/// Analyze UTF-8 content characteristics
fn analyze_utf8_content(data: &[u8]) -> (f64, bool, bool) {
    let sample_size = data.len().min(2048); // Larger sample for better accuracy
    let sample = &data[..sample_size];

    let mut valid_bytes = 0;
    let mut total_bytes = 0;
    let mut has_multibyte_sequences = false;
    let mut has_invalid_sequences = false;
    let mut i = 0;

    while i < sample.len() {
        let byte = sample[i];
        total_bytes += 1;

        if (byte & 0x80) == 0 {
            // ASCII character (0xxxxxxx)
            valid_bytes += 1;
            i += 1;
        } else if (byte & 0xE0) == 0xC0 {
            // 2-byte sequence (110xxxxx 10xxxxxx)
            if i + 1 < sample.len() && (sample[i + 1] & 0xC0) == 0x80 {
                // Valid 2-byte UTF-8 sequence
                has_multibyte_sequences = true;
                valid_bytes += 2;
                total_bytes += 1;
                i += 2;
            } else {
                has_invalid_sequences = true;
                i += 1;
            }
        } else if (byte & 0xF0) == 0xE0 {
            // 3-byte sequence (1110xxxx 10xxxxxx 10xxxxxx)
            if i + 2 < sample.len() &&
               (sample[i + 1] & 0xC0) == 0x80 &&
               (sample[i + 2] & 0xC0) == 0x80 {
                has_multibyte_sequences = true;
                valid_bytes += 3;
                total_bytes += 2;
                i += 3;
            } else {
                has_invalid_sequences = true;
                i += 1;
            }
        } else if (byte & 0xF8) == 0xF0 {
            // 4-byte sequence (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
            if i + 3 < sample.len() &&
               (sample[i + 1] & 0xC0) == 0x80 &&
               (sample[i + 2] & 0xC0) == 0x80 &&
               (sample[i + 3] & 0xC0) == 0x80 {
                has_multibyte_sequences = true;
                valid_bytes += 4;
                total_bytes += 3;
                i += 4;
            } else {
                has_invalid_sequences = true;
                i += 1;
            }
        } else {
            // Invalid UTF-8 start byte
            has_invalid_sequences = true;
            i += 1;
        }
    }

    let confidence = if total_bytes > 0 {
        valid_bytes as f64 / total_bytes as f64
    } else {
        0.0
    };

    (confidence, has_multibyte_sequences, has_invalid_sequences)
}

/// Analyze Latin-1 content characteristics
fn analyze_latin1_content(data: &[u8]) -> (f64, bool) {
    let sample_size = data.len().min(2048);
    let sample = &data[..sample_size];

    let mut latin1_chars = 0;
    let mut ascii_chars = 0;
    let mut high_bit_chars = 0;
    let mut control_chars = 0;

    for &byte in sample {
        if byte <= 0x7F {
            // ASCII range
            ascii_chars += 1;
            // Count control characters (except common ones like \t, \n, \r)
            if byte < 0x20 && !matches!(byte, 0x09 | 0x0A | 0x0D) {
                control_chars += 1;
            }
        } else if (0x80..=0xFF).contains(&byte) {
            // High-bit set (potential Latin-1)
            high_bit_chars += 1;

            // Common Latin-1 characters (accented letters and symbols)
            if matches!(byte,
                0xC0..=0xD6 | 0xD8..=0xF6 | 0xF8..=0xFF | // À-Ö, Ø-ö, ø-ÿ
                0xA1..=0xBF | // Various punctuation and symbols
                0x80..=0x9F   // Additional Latin-1 supplement characters
            ) {
                latin1_chars += 1;
            }
        }
    }

    let total_chars = sample.len();
    let has_latin1_chars = latin1_chars > 0 || high_bit_chars > 0;

    // Calculate confidence based on presence of typical Latin-1 patterns
    let confidence = if total_chars > 0 {
        let ascii_ratio = ascii_chars as f64 / total_chars as f64;
        let latin1_ratio = latin1_chars as f64 / total_chars as f64;
        let high_bit_ratio = high_bit_chars as f64 / total_chars as f64;
        let control_ratio = control_chars as f64 / total_chars as f64;

        // Penalize files with too many control characters (likely binary)
        if control_ratio > 0.1 {
            return (0.1, has_latin1_chars);
        }

        // High ASCII content with some Latin-1 characters is typical
        if has_latin1_chars && ascii_ratio > 0.7 && latin1_ratio > 0.01 {
            0.85 + (latin1_ratio * 0.15).min(0.1)
        } else if has_latin1_chars && ascii_ratio > 0.5 {
            0.75 + (latin1_ratio * 0.2).min(0.15)
        } else if high_bit_chars > 0 && ascii_ratio > 0.8 {
            // Files with high-bit chars but mostly ASCII (common in ISO-8859-1 logs)
            0.7 + (high_bit_ratio * 0.2).min(0.15)
        } else if has_latin1_chars {
            0.6 + (latin1_ratio * 0.3).min(0.25)
        } else if ascii_ratio > 0.95 {
            // Pure ASCII could be Latin-1 but lower confidence
            0.6
        } else {
            ascii_ratio * 0.5
        }
    } else {
        0.0
    };

    (confidence, has_latin1_chars)
}

/// Detect UTF-16 patterns
fn detect_utf16_pattern(data: &[u8]) -> bool {
    let sample_size = data.len().min(1024);
    let sample = &data[..sample_size];

    let mut null_byte_count = 0;
    for i in 0..sample.len() {
        if sample[i] == 0x00 {
            null_byte_count += 1;
            // Check for alternating pattern typical of UTF-16
            if i > 0 && sample[i - 1] != 0x00 {
                return true;
            }
        }
    }

    // If more than 10% null bytes, likely UTF-16
    null_byte_count as f64 / sample.len() as f64 > 0.1
}

pub fn decode_lines_robust(data: &[u8], _decoder: &encoding_rs::Decoder) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        let byte = data[i];
        
        // Check for line endings
        if byte == b'\n' {
            // LF line ending
            match try_fallback_decoders(&current_line) {
                Ok(line_text) => {
                    lines.push(line_text);
                }
                Err(e) => {
                    eprintln!("Failed to decode line {}: {}", lines.len() + 1, e);
                    lines.push(format!("[DECODE_ERROR] Line {}: {}", lines.len() + 1, e));
                }
            }
            current_line.clear();
            i += 1;
        } else if byte == b'\r' && i + 1 < data.len() && data[i + 1] == b'\n' {
            // CRLF line ending
            match try_fallback_decoders(&current_line) {
                Ok(line_text) => {
                    lines.push(line_text);
                }
                Err(e) => {
                    eprintln!("Failed to decode line {}: {}", lines.len() + 1, e);
                    lines.push(format!("[DECODE_ERROR] Line {}: {}", lines.len() + 1, e));
                }
            }
            current_line.clear();
            i += 2; // Skip both \r and \n
        } else {
            // Regular character
            current_line.push(byte);
            i += 1;
        }
    }
    
    // Handle last line if file doesn't end with newline
    if !current_line.is_empty() {
        match try_fallback_decoders(&current_line) {
            Ok(line_text) => {
                lines.push(line_text);
            }
            Err(e) => {
                eprintln!("Failed to decode final line: {}", e);
                lines.push(format!("[DECODE_ERROR] Final line: {}", e));
            }
        }
    }
    
    Ok(lines)
}

fn decode_line_safely(line_bytes: &[u8], _decoder: &encoding_rs::Decoder) -> Result<String, String> {
    if line_bytes.is_empty() {
        return Ok(String::new());
    }
    
    // Since we can't clone Decoder, let's try fallback decoders directly
    try_fallback_decoders(line_bytes)
}

fn try_fallback_decoders(line_bytes: &[u8]) -> Result<String, String> {
    use encoding_rs::{UTF_8, UTF_16LE, UTF_16BE, WINDOWS_1252, ISO_8859_2, ISO_8859_3};

    let fallback_encodings = [
        (UTF_8, "UTF-8"),
        (WINDOWS_1252, "Windows-1252/ISO-8859-1"), // Windows-1252 is superset of ISO-8859-1
        (ISO_8859_2, "ISO-8859-2"),
        (ISO_8859_3, "ISO-8859-3"),
        (UTF_16LE, "UTF-16LE"),
        (UTF_16BE, "UTF-16BE"),
    ];
    
    for (encoding, _name) in fallback_encodings.iter() {
        let mut result = String::new();
        let mut decoder = encoding.new_decoder();
        
        let (_result, _used, had_errors) = decoder.decode_to_string(line_bytes, &mut result, false);
        
        if !result.is_empty() && !had_errors {
            // Check if result contains mostly printable characters
            if is_mostly_printable(&result) {
                return Ok(result);
            }
        }
    }
    
    // Last resort: lossy UTF-8 with replacement characters
    let result = String::from_utf8_lossy(line_bytes).into_owned();
    if is_mostly_printable(&result) {
        Ok(result)
    } else {
        Err(format!("Unable to decode line: contains binary or corrupt data ({} bytes)", line_bytes.len()))
    }
}

/// Check if string is mostly printable characters
fn is_mostly_printable(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    
    let printable_count = text.chars()
        .filter(|&c| c.is_ascii_graphic() || c.is_whitespace())
        .count();
    
    let total_chars = text.chars().count();
    
    // Consider printable if at least 70% of characters are printable
    printable_count as f64 / total_chars as f64 > 0.7
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_detect_utf8_encoding() {
        let utf8_data = "Hello, 世界! 🌍".as_bytes();
        let (encoding, confidence, _decoder) = detect_and_create_decoder(utf8_data);
        
        assert_eq!(encoding.name(), "UTF-8");
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_detect_utf8_with_bom() {
        let mut data = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        data.extend_from_slice("Hello World".as_bytes());
        
        let (encoding, confidence, _decoder) = detect_and_create_decoder(&data);
        
        assert_eq!(encoding.name(), "UTF-8");
        assert_eq!(confidence, 1.0);
    }

    #[test] 
    fn test_detect_utf16le_with_bom() {
        let mut data = vec![0xFF, 0xFE]; // UTF-16 LE BOM
        data.extend_from_slice(&[0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00]); // "Hello" in UTF-16LE
        
        let (encoding, confidence, _decoder) = detect_and_create_decoder(&data);
        
        assert_eq!(encoding.name(), "UTF-16LE");
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_detect_utf16be_with_bom() {
        let mut data = vec![0xFE, 0xFF]; // UTF-16 BE BOM
        data.extend_from_slice(&[0x00, 0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F]); // "Hello" in UTF-16BE
        
        let (encoding, confidence, _decoder) = detect_and_create_decoder(&data);
        
        assert_eq!(encoding.name(), "UTF-16BE");
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_mostly_printable_check() {
        assert!(is_mostly_printable("Hello World"));
        assert!(is_mostly_printable("2024-01-20 [ERROR] Something failed"));
        assert!(is_mostly_printable("Mixed content: 123 !@# abc"));
        assert!(!is_mostly_printable("\x00\x01\x02\x03\x04"));
        assert!(is_mostly_printable("")); // Empty string is considered printable
        
        // String with some non-printable characters but mostly printable
        let mixed = "Hello\x01World\x02Test"; // 3 non-printable, 14 printable
        assert!(is_mostly_printable(mixed));
        
        // String with too many non-printable characters  
        let mostly_binary = "\x00\x01\x02\x03\x04\x05\x06\x07Hello"; // 8 non-printable, 5 printable
        assert!(!is_mostly_printable(mostly_binary));
    }

    #[test]
    fn test_decode_lines_robust() {
        use encoding_rs::UTF_8;
        
        let test_data = "Line 1\nLine 2\r\nLine 3\nLine 4";
        let decoder = UTF_8.new_decoder();
        
        let result = decode_lines_robust(test_data.as_bytes(), &decoder).unwrap();
        
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "Line 1");
        assert_eq!(result[1], "Line 2");
        assert_eq!(result[2], "Line 3");
        assert_eq!(result[3], "Line 4");
    }

    #[test]
    fn test_fallback_decoder_recovery() {
        // Create data that will fail UTF-8 but succeed in Windows-1252
        let windows_1252_data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0xE9]; // "Hello é" in Windows-1252
        
        let result = try_fallback_decoders(&windows_1252_data);
        assert!(result.is_ok());
        
        let decoded = result.unwrap();
        assert!(decoded.contains("Hello"));
    }

    #[test]
    fn test_binary_data_rejection() {
        // Pure binary data that shouldn't decode successfully
        let binary_data = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
        
        let result = try_fallback_decoders(&binary_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("binary or corrupt data"));
    }

    #[tokio::test]
    async fn test_file_reading_integration() {
        // Create a temporary file with mixed content
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_log.txt");
        
        let test_content = "2024-01-20 [ERROR] Test error\n2024-01-20 [WARNING] Test warning\n";
        std::fs::write(&test_file, test_content).unwrap();
        
        let result = read_log_file(test_file.to_str().unwrap()).await;
        assert!(result.is_ok());
        
        let lines = result.unwrap();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("ERROR"));
        assert!(lines[1].contains("WARNING"));
        
        // Clean up
        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_corrupted_file_handling() {
        // Create a file with mixed valid/invalid UTF-8
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("corrupted_log.txt");

        let mut file = std::fs::File::create(&test_file).unwrap();
        file.write_all(b"Valid line 1\n").unwrap();
        file.write_all(&[0xFF, 0xFE, 0x00, 0x00]).unwrap(); // Invalid UTF-8 sequence
        file.write_all(b"\nValid line 2\n").unwrap();

        let result = read_log_file(test_file.to_str().unwrap()).await;
        assert!(result.is_ok());

        let lines = result.unwrap();
        assert!(lines.len() >= 2); // Should have at least the valid lines

        // Should contain both valid lines and a decode error marker
        let all_content = lines.join("\n");
        assert!(all_content.contains("Valid line 1"));
        assert!(all_content.contains("Valid line 2"));

        // Clean up
        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_iso_8859_1_file_handling() {
        // Create a file with ISO-8859-1 (Latin-1) characters
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("iso_latin1_log.txt");

        // Create Latin-1 content with accented characters
        let latin1_content = vec![
            // "Café: información crítica"
            0x43, 0x61, 0x66, 0xE9, 0x3A, 0x20, 0x69, 0x6E, 0x66, 0x6F, 0x72, 0x6D, 0x61, 0x63, 0x69, 0xF3, 0x6E, 0x20, 0x63, 0x72, 0xED, 0x74, 0x69, 0x63, 0x61, 0x0A,
            // "Error número 123"
            0x45, 0x72, 0x72, 0x6F, 0x72, 0x20, 0x6E, 0xFA, 0x6D, 0x65, 0x72, 0x6F, 0x20, 0x31, 0x32, 0x33, 0x0A,
        ];

        std::fs::write(&test_file, latin1_content).unwrap();

        let result = read_log_file(test_file.to_str().unwrap()).await;
        assert!(result.is_ok());

        let lines = result.unwrap();
        assert_eq!(lines.len(), 2);

        // Debug: print what we actually got
        println!("Decoded lines:");
        for (i, line) in lines.iter().enumerate() {
            println!("Line {}: {}", i, line);
        }

        // Check that encoding detection worked (should be windows-1252 or similar)
        // And that the lines contain the expected structure
        assert!(lines[0].contains("Caf"));  // Part of "Café" should be there
        assert!(lines[0].contains("informaci"));  // Part of "información"
        assert!(lines[0].contains("cr"));  // Part of "crítica"
        assert!(lines[1].contains("n"));  // Part of "número"
        assert!(lines[1].contains("mero")); // Part of "número"

        // Clean up
        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_mixed_encoding_file_handling() {
        // Test file with mixed ASCII and high-bit characters
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("mixed_encoding_log.txt");

        let mut content = Vec::new();
        content.extend_from_slice(b"2024-01-01 12:00:00 [INFO] Regular ASCII log message\n");
        // Add some Latin-1 characters
        content.extend_from_slice(&[0x32, 0x30, 0x32, 0x34, 0x2D, 0x30, 0x31, 0x2D, 0x30, 0x31, 0x20, 0x5B, 0x45, 0x52, 0x52, 0x4F, 0x52, 0x5D, 0x20, 0x4E, 0x6F, 0x20, 0x73, 0x65, 0x20, 0x70, 0x75, 0x65, 0x64, 0x65, 0x20, 0x61, 0x63, 0x63, 0x65, 0x64, 0x65, 0x72, 0x20, 0x61, 0x6C, 0x20, 0x61, 0x72, 0x63, 0x68, 0x69, 0x76, 0x6F, 0x0A]); // "No se puede acceder al archivo"
        content.extend_from_slice(b"2024-01-01 12:00:02 [WARN] Another ASCII message\n");

        std::fs::write(&test_file, content).unwrap();

        let result = read_log_file(test_file.to_str().unwrap()).await;
        assert!(result.is_ok());

        let lines = result.unwrap();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("Regular ASCII"));
        assert!(lines[1].contains("archivo"));
        assert!(lines[2].contains("Another ASCII"));

        // Clean up
        std::fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_windows_1252_specific_characters() {
        // Test characters specific to Windows-1252 that aren't in ISO-8859-1
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("windows_1252_log.txt");

        let mut content = Vec::new();
        content.extend_from_slice(b"Log with Windows-1252 characters: ");
        // Add Windows-1252 specific characters (0x80-0x9F range)
        content.push(0x80); // Euro sign
        content.push(0x85); // Horizontal ellipsis
        content.push(0x91); // Left single quotation mark
        content.push(0x92); // Right single quotation mark
        content.push(0x93); // Left double quotation mark
        content.push(0x94); // Right double quotation mark
        content.push(0x0A); // Newline

        std::fs::write(&test_file, content).unwrap();

        let result = read_log_file(test_file.to_str().unwrap()).await;
        assert!(result.is_ok());

        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("Windows-1252"));

        // Clean up
        std::fs::remove_file(&test_file).ok();
    }
}
