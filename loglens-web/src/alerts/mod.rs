/// Phase 6.3: Alert & Notification System
/// 
/// This module implements a comprehensive alert and notification system that builds
/// upon the streaming infrastructure from Phase 6.1 and dashboard from Phase 6.2.

pub mod engine;
pub mod notifications;
pub mod rules;
pub mod channels;

pub use engine::{AlertEngine, AlertManager};
pub use notifications::{NotificationManager, AlertNotification};
pub use rules::{AlertRule, AlertCondition, AlertSeverity};
pub use channels::{NotificationChannel, ChannelType};