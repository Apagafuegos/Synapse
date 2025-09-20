//! Process Monitoring Module
//! 
//! Provides real-time process execution, log capture, and AI analysis integration
//! for automatic log analysis without manual file uploads.

pub mod executor;
pub mod buffer;
pub mod triggers;

pub use executor::*;
pub use buffer::*;
pub use triggers::*;