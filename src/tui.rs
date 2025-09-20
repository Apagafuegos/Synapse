use crate::model::LogEntry;
use crate::analytics::LogAnalytics;
use crate::filters::FilterChain;
use crate::advanced_filters::AdvancedFilterChain;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self},
    cursor,
};
use std::io::{self, Stdout};
use std::time::Duration;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationMode {
    LogView,
    AnalyticsView,
    FilterBuilder,
    Help,
}

pub struct TuiApp {
    pub log_entries: Vec<LogEntry>,
    pub filtered_entries: Vec<LogEntry>,
    pub current_filter: String,
    pub selected_entry: usize,
    pub scroll_offset: usize,
    pub analytics: LogAnalytics,
    pub mode: ApplicationMode,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub current_search_result: usize,
    pub show_help: bool,
    pub status_message: String,
    pub filter_chain: FilterChain,
    pub advanced_filter_chain: Option<AdvancedFilterChain>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            log_entries: Vec::new(),
            filtered_entries: Vec::new(),
            current_filter: String::new(),
            selected_entry: 0,
            scroll_offset: 0,
            analytics: LogAnalytics::new(),
            mode: ApplicationMode::LogView,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_result: 0,
            show_help: false,
            status_message: String::new(),
            filter_chain: FilterChain::new(),
            advanced_filter_chain: None,
        }
    }
}

impl TuiApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_log_entry(&mut self, entry: LogEntry) {
        self.log_entries.push(entry.clone());
        self.analytics.analyze_entry(&entry);
        self.apply_filters();
    }

    pub fn apply_filters(&mut self) {
        self.filtered_entries = self.log_entries.iter()
            .filter(|entry| {
                let basic_match = self.filter_chain.matches(entry);
                let advanced_match = if let Some(chain) = self.advanced_filter_chain.as_mut() {
                    chain.matches(entry)
                } else {
                    true
                };
                
                basic_match && advanced_match
            })
            .cloned()
            .collect();

        if !self.filtered_entries.is_empty() && self.selected_entry >= self.filtered_entries.len() {
            self.selected_entry = self.filtered_entries.len() - 1;
        }

        self.update_status_message();
    }

    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_results.clear();
        self.current_search_result = 0;

        if query.is_empty() {
            return;
        }

        for (i, entry) in self.filtered_entries.iter().enumerate() {
            if entry.message.to_lowercase().contains(&query.to_lowercase()) ||
               entry.raw_line.to_lowercase().contains(&query.to_lowercase()) {
                self.search_results.push(i);
            }
        }

        if !self.search_results.is_empty() {
            self.selected_entry = self.search_results[0];
            self.scroll_to_selected();
        }

        self.update_status_message();
    }

    pub fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_result = (self.current_search_result + 1) % self.search_results.len();
            self.selected_entry = self.search_results[self.current_search_result];
            self.scroll_to_selected();
        }
    }

    pub fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_result = (self.current_search_result + self.search_results.len() - 1) % self.search_results.len();
            self.selected_entry = self.search_results[self.current_search_result];
            self.scroll_to_selected();
        }
    }

    pub fn scroll_to_selected(&mut self) {
        let visible_lines = 20; // Approximate visible lines in log view
        if self.selected_entry < self.scroll_offset {
            self.scroll_offset = self.selected_entry;
        } else if self.selected_entry >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.selected_entry - visible_lines + 1;
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.filtered_entries.is_empty() {
            return;
        }

        let new_pos = self.selected_entry as isize + delta;
        if new_pos >= 0 && new_pos < self.filtered_entries.len() as isize {
            self.selected_entry = new_pos as usize;
            self.scroll_to_selected();
        }
    }

    pub fn update_status_message(&mut self) {
        let total = self.log_entries.len();
        let filtered = self.filtered_entries.len();
        let search_count = self.search_results.len();
        
        self.status_message = if !self.search_query.is_empty() {
            format!("Total: {} | Filtered: {} | Search: {}/{}", total, filtered, self.current_search_result + 1, search_count)
        } else {
            format!("Total: {} | Filtered: {} | Selected: {}/{}", total, filtered, self.selected_entry + 1, filtered)
        };
    }

    pub fn get_selected_entry(&self) -> Option<&LogEntry> {
        self.filtered_entries.get(self.selected_entry)
    }
}

pub struct TuiApplication {
    app: TuiApp,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TuiApplication {
    pub fn new() -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        Ok(Self {
            app: TuiApp::new(),
            terminal,
        })
    }

    pub fn app(&mut self) -> &mut TuiApp {
        &mut self.app
    }

    pub fn run(&mut self) -> Result<()> {
        self.setup_terminal()?;
        self.run_event_loop()?;
        self.restore_terminal()?;
        Ok(())
    }

    fn setup_terminal(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        self.terminal.clear()?;
        Ok(())
    }

    fn restore_terminal(&mut self) -> Result<()> {
        execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn run_event_loop(&mut self) -> Result<()> {
        loop {
            self.draw()?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if self.handle_key_event(key_event)? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool> {
        match key_event.code {
            KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(true); // Exit
            }
            KeyCode::Char('q') => {
                return Ok(true); // Exit
            }
            KeyCode::Char('h') => {
                self.app.show_help = !self.app.show_help;
            }
            KeyCode::Char('f') => {
                self.app.mode = ApplicationMode::FilterBuilder;
            }
            KeyCode::Char('a') => {
                self.app.mode = ApplicationMode::AnalyticsView;
            }
            KeyCode::Char('l') => {
                self.app.mode = ApplicationMode::LogView;
            }
            KeyCode::Char('/') => {
                // Start search
                self.app.search_query.clear();
                self.app.search_results.clear();
            }
            KeyCode::Char('n') => {
                self.app.next_search_result();
            }
            KeyCode::Char('p') => {
                self.app.prev_search_result();
            }
            KeyCode::Esc => {
                self.app.mode = ApplicationMode::LogView;
                self.app.show_help = false;
            }
            KeyCode::Up => {
                self.app.move_selection(-1);
            }
            KeyCode::Down => {
                self.app.move_selection(1);
            }
            KeyCode::PageUp => {
                self.app.move_selection(-10);
            }
            KeyCode::PageDown => {
                self.app.move_selection(10);
            }
            KeyCode::Home => {
                if !self.app.filtered_entries.is_empty() {
                    self.app.selected_entry = 0;
                    self.app.scroll_offset = 0;
                }
            }
            KeyCode::End => {
                if !self.app.filtered_entries.is_empty() {
                    self.app.selected_entry = self.app.filtered_entries.len() - 1;
                    self.app.scroll_to_selected();
                }
            }
            KeyCode::Enter => {
                if let Some(entry) = self.app.get_selected_entry() {
                    // Show detailed view or copy to clipboard
                    self.app.status_message = format!("Selected: {}", entry.message.chars().take(50).collect::<String>());
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn draw(&mut self) -> Result<()> {
        let app = &self.app;
        
        self.terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(1),     // Main content
                    Constraint::Length(3),  // Status bar
                ])
                .split(size);

            Self::draw_header_static(f, chunks[0]);
            
            match app.mode {
                ApplicationMode::LogView => Self::draw_log_view_static(f, chunks[1], app),
                ApplicationMode::AnalyticsView => Self::draw_analytics_view_static(f, chunks[1], app),
                ApplicationMode::FilterBuilder => Self::draw_filter_builder_static(f, chunks[1], app),
                ApplicationMode::Help => Self::draw_help_view_static(f, chunks[1]),
            }
            
            Self::draw_status_bar_static(f, chunks[2], app);
        })?;
        Ok(())
    }

    fn draw_header_static(f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let header_text = Text::from(Spans::from(vec![
            Span::styled("LogLens", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" - Interactive Log Analyzer v0.1.0"),
        ]));
        
        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).title("Header"));
        
        f.render_widget(header, area);
    }

    fn draw_log_view_static(f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, app: &TuiApp) {
        let log_items: Vec<ListItem> = app.filtered_entries
            .iter()
            .skip(app.scroll_offset)
            .take(area.height as usize - 2)
            .enumerate()
            .map(|(i, entry)| {
                let actual_index = app.scroll_offset + i;
                let is_selected = actual_index == app.selected_entry;
                let is_search_result = app.search_results.contains(&actual_index);
                
                let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                let level = entry.level.to_string();
                let message = entry.message.chars().take(80).collect::<String>();
                
                let mut style = Style::default();
                if is_selected {
                    style = style.fg(Color::Yellow).add_modifier(Modifier::REVERSED);
                } else if is_search_result {
                    style = style.fg(Color::Green);
                }
                
                // Color by log level
                match entry.level {
                    crate::model::LogLevel::Error => style = style.fg(Color::Red),
                    crate::model::LogLevel::Warn => style = style.fg(Color::Yellow),
                    crate::model::LogLevel::Info => style = style.fg(Color::Blue),
                    crate::model::LogLevel::Debug => style = style.fg(Color::Gray),
                    crate::model::LogLevel::Trace => style = style.fg(Color::DarkGray),
                    _ => {}
                }
                
                let text = format!("[{}] {} {}", timestamp, level, message);
                ListItem::new(Span::styled(text, style))
            })
            .collect();

        let log_list = List::new(log_items)
            .block(Block::default().borders(Borders::ALL).title("Log Entries"));
        
        f.render_widget(log_list, area);
    }

    fn draw_analytics_view_static(f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, app: &TuiApp) {
        let analytics_data = app.analytics.get_visualization_data();
        
        let mut analytics_text = String::new();
        analytics_text.push_str("=== Analytics Summary ===\n\n");
        
        // Timeline info
        if let (Some(first), Some(last)) = (analytics_data.timeline.first(), analytics_data.timeline.last()) {
            analytics_text.push_str(&format!("Time Range: {} to {}\n", first.timestamp, last.timestamp));
            analytics_text.push_str(&format!("Total Entries: {}\n", 
                analytics_data.timeline.iter().map(|p| p.count).sum::<usize>()));
            analytics_text.push_str(&format!("Peak Rate: {} entries/minute\n", 
                analytics_data.timeline.iter().map(|p| p.count).max().unwrap_or(0)));
        }
        
        // Level distribution
        analytics_text.push_str("\n--- Level Distribution ---\n");
        let total: usize = analytics_data.level_distribution.values().sum();
        for (level, count) in &analytics_data.level_distribution {
            let percentage = (*count as f64 / total as f64) * 100.0;
            analytics_text.push_str(&format!("  {}: {} ({:.1}%)\n", level, count, percentage));
        }
        
        // Top patterns
        analytics_text.push_str("\n--- Top Patterns ---\n");
        for (i, pattern) in analytics_data.patterns.iter().take(5).enumerate() {
            analytics_text.push_str(&format!("  {}. {} ({} times)\n", i + 1, pattern.pattern, pattern.frequency));
        }
        
        // Anomalies
        if !analytics_data.anomalies.is_empty() {
            analytics_text.push_str("\n--- Anomalies Detected ---\n");
            for (i, anomaly) in analytics_data.anomalies.iter().take(3).enumerate() {
                analytics_text.push_str(&format!("  {}. Score: {:.2}\n", i + 1, anomaly.score));
                analytics_text.push_str(&format!("     {}\n", anomaly.message));
            }
        }
        
        let analytics_paragraph = Paragraph::new(analytics_text)
            .block(Block::default().borders(Borders::ALL).title("Analytics"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(analytics_paragraph, area);
    }

    fn draw_filter_builder_static(f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, app: &TuiApp) {
        let filter_text = format!(
            "Current Filter: {}\n\n\
            Filter Options:\n\
            - level: Filter by log level (ERROR, WARN, INFO, DEBUG)\n\
            - time: Filter by time range (start:end)\n\
            - pattern: Filter by regex pattern\n\
            - query: Advanced query language\n\n\
            Examples:\n\
            level=ERROR\n\
            time=2023-01-01:2023-01-02\n\
            pattern=database.*error\n\
            query=\"level==ERROR AND message~timeout\"\n\n\
            Press ESC to return to log view",
            app.current_filter
        );
        
        let filter_paragraph = Paragraph::new(filter_text)
            .block(Block::default().borders(Borders::ALL).title("Filter Builder"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(filter_paragraph, area);
    }

    fn draw_help_view_static(f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let help_text = "
=== LogLens TUI Help ===

Navigation:
  ↑/↓           - Move selection up/down
  PageUp/PageDown - Move 10 entries at a time
  Home/End      - Jump to first/last entry
  Enter         - View selected entry details

Search:
  /              - Start search
  n              - Next search result
  p              - Previous search result

Views:
  l              - Log view (default)
  a              - Analytics view
  f              - Filter builder
  h              - Toggle this help

Other:
  q or Ctrl+C    - Quit application
  Esc            - Return to log view

Filters:
  The application supports basic and advanced filtering.
  Use the filter builder to construct complex queries.

Analytics:
  View real-time analytics including level distribution,
  pattern analysis, and anomaly detection.
        ";
        
        let help_paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(help_paragraph, area);
    }

    #[allow(dead_code)]
    fn draw_analytics_view_with_data(&self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, analytics_data: &crate::analytics::VisualizationData) {
        let mut analytics_text = String::new();
        analytics_text.push_str("=== Analytics Summary ===\n\n");
        
        // Timeline info
        if let (Some(first), Some(last)) = (analytics_data.timeline.first(), analytics_data.timeline.last()) {
            analytics_text.push_str(&format!("Time Range: {} to {}\n", first.timestamp, last.timestamp));
            analytics_text.push_str(&format!("Total Entries: {}\n", 
                analytics_data.timeline.iter().map(|p| p.count).sum::<usize>()));
            analytics_text.push_str(&format!("Peak Rate: {} entries/minute\n", 
                analytics_data.timeline.iter().map(|p| p.count).max().unwrap_or(0)));
        }
        
        // Level distribution
        analytics_text.push_str("\n--- Level Distribution ---\n");
        let total: usize = analytics_data.level_distribution.values().sum();
        for (level, count) in &analytics_data.level_distribution {
            let percentage = (*count as f64 / total as f64) * 100.0;
            analytics_text.push_str(&format!("  {}: {} ({:.1}%)\n", level, count, percentage));
        }
        
        // Top patterns
        analytics_text.push_str("\n--- Top Patterns ---\n");
        for (i, pattern) in analytics_data.patterns.iter().take(5).enumerate() {
            analytics_text.push_str(&format!("  {}. {} ({} times)\n", i + 1, pattern.pattern, pattern.frequency));
        }
        
        // Anomalies
        if !analytics_data.anomalies.is_empty() {
            analytics_text.push_str("\n--- Anomalies Detected ---\n");
            for (i, anomaly) in analytics_data.anomalies.iter().take(3).enumerate() {
                analytics_text.push_str(&format!("  {}. Score: {:.2}\n", i + 1, anomaly.score));
                analytics_text.push_str(&format!("     {}\n", anomaly.message));
            }
        }
        
        let analytics_paragraph = Paragraph::new(analytics_text)
            .block(Block::default().borders(Borders::ALL).title("Analytics"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(analytics_paragraph, area);
    }

    #[allow(dead_code)]
    fn draw_filter_builder_with_filter(&self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, current_filter: &str) {
        let filter_text = format!(
            "Current Filter: {}\n\n\
            Filter Options:\n\
            - level: Filter by log level (ERROR, WARN, INFO, DEBUG)\n\
            - time: Filter by time range (start:end)\n\
            - pattern: Filter by regex pattern\n\
            - query: Advanced query language\n\n\
            Examples:\n\
            level=ERROR\n\
            time=2023-01-01:2023-01-02\n\
            pattern=database.*error\n\
            query=\"level==ERROR AND message~timeout\"\n\n\
            Press ESC to return to log view",
            current_filter
        );
        
        let filter_paragraph = Paragraph::new(filter_text)
            .block(Block::default().borders(Borders::ALL).title("Filter Builder"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(filter_paragraph, area);
    }

    fn draw_status_bar_static(f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect, app: &TuiApp) {
        let status_text = format!(
            "{} | Mode: {:?} | Entries: {} | h:help | q:quit",
            app.status_message,
            app.mode,
            app.filtered_entries.len()
        );
        
        let status_paragraph = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        
        f.render_widget(status_paragraph, area);
    }

    #[allow(dead_code)]
    fn draw_status_bar(&self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let status_text = format!(
            "{} | Mode: {:?} | Entries: {} | h:help | q:quit",
            self.app.status_message,
            self.app.mode,
            self.app.filtered_entries.len()
        );
        
        let status_paragraph = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        
        f.render_widget(status_paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LogLevel;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_tui_app_creation() {
        let app = TuiApp::new();
        assert_eq!(app.selected_entry, 0);
        assert_eq!(app.scroll_offset, 0);
        assert!(app.log_entries.is_empty());
        assert!(app.filtered_entries.is_empty());
    }

    #[test]
    fn test_add_log_entry() {
        let mut app = TuiApp::new();
        let entry = LogEntry::new(
            Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
            LogLevel::Info,
            "Test message".to_string(),
            "raw line".to_string(),
        );
        
        app.add_log_entry(entry);
        assert_eq!(app.log_entries.len(), 1);
        assert_eq!(app.filtered_entries.len(), 1);
    }

    #[test]
    fn test_search_functionality() {
        let mut app = TuiApp::new();
        
        // Add test entries
        for i in 0..5 {
            let entry = LogEntry::new(
                Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, i).unwrap(),
                LogLevel::Info,
                format!("Test message {}", i),
                format!("raw line {}", i),
            );
            app.add_log_entry(entry);
        }
        
        app.search("message 2");
        assert_eq!(app.search_results.len(), 1);
        assert_eq!(app.selected_entry, 2); // Should select the matching entry
    }

    #[test]
    fn test_navigation() {
        let mut app = TuiApp::new();
        
        // Add test entries
        for i in 0..10 {
            let entry = LogEntry::new(
                Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, i).unwrap(),
                LogLevel::Info,
                format!("Test message {}", i),
                format!("raw line {}", i),
            );
            app.add_log_entry(entry);
        }
        
        app.move_selection(5);
        assert_eq!(app.selected_entry, 5);
        
        app.move_selection(-2);
        assert_eq!(app.selected_entry, 3);
        
        app.move_selection(-10); // Should not go below 0
        assert_eq!(app.selected_entry, 0);
    }
}