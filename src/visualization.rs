use crate::analytics::VisualizationData;
use serde::{Deserialize, Serialize};

#[cfg(feature = "visualization")]
use plotters::style::RGBColor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub colors: Vec<String>,
}

pub struct LogVisualizer {
    #[allow(dead_code)]
    config: ChartConfig,
}

impl LogVisualizer {
    pub fn new() -> Self {
        Self {
            config: ChartConfig {
                width: 800,
                height: 400,
                title: "Log Analysis".to_string(),
                colors: vec![
                    "#3498db".to_string(), // Blue
                    "#e74c3c".to_string(), // Red
                    "#2ecc71".to_string(), // Green
                    "#f39c12".to_string(), // Orange
                    "#9b59b6".to_string(), // Purple
                ],
            },
        }
    }

    pub fn generate_timeline_chart(&self, _data: &VisualizationData) -> Result<String, String> {
        #[cfg(feature = "visualization")]
        {
            use plotters::prelude::*;

            let mut buffer = Vec::new();
            {
                let root =
                    SVGBackend::with_buffer(&mut buffer, (self.config.width, self.config.height))
                        .into_drawing_area();

                root.fill(&WHITE)
                    .map_err(|e| format!("Drawing error: {}", e))?;

                let mut chart = ChartBuilder::on(&root)
                    .caption(&self.config.title, ("sans-serif", 20).into_font())
                    .margin(10)
                    .x_label_area_size(40)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        data.timeline
                            .first()
                            .map(|p| p.timestamp)
                            .unwrap_or_default()
                            ..data
                                .timeline
                                .last()
                                .map(|p| p.timestamp)
                                .unwrap_or_default(),
                        0..data.timeline.iter().map(|p| p.count).max().unwrap_or(1),
                    )
                    .map_err(|e| format!("Chart building error: {}", e))?;

                // Configure mesh
                chart
                    .configure_mesh()
                    .x_desc("Time")
                    .y_desc("Log Count")
                    .draw()
                    .map_err(|e| format!("Mesh drawing error: {}", e))?;

                // Draw timeline
                if !data.timeline.is_empty() {
                    let points: Vec<_> = data
                        .timeline
                        .iter()
                        .map(|p| (p.timestamp, p.count))
                        .collect();

                    chart
                        .draw_series(LineSeries::new(
                            points,
                            &self.config.colors[0].parse::<RGBColor>().unwrap_or(BLUE),
                        ))
                        .map_err(|e| format!("Line drawing error: {}", e))?
                        .label("Log Volume");
                }

                // Draw level breakdown if available
                for (i, (level, color)) in ["ERROR", "WARN", "INFO", "DEBUG"]
                    .iter()
                    .zip(self.config.colors.iter().skip(1))
                    .enumerate()
                {
                    let level_points: Vec<_> = data
                        .timeline
                        .iter()
                        .filter_map(|p| {
                            p.level_breakdown
                                .get(*level)
                                .map(|&count| (p.timestamp, count))
                        })
                        .collect();

                    if !level_points.is_empty() {
                        chart
                            .draw_series(LineSeries::new(
                                level_points,
                                &color.parse::<RGBColor>().unwrap_or(GREEN),
                            ))
                            .map_err(|e| format!("Level line drawing error: {}", e))?
                            .label(*level);
                    }
                }

                // Configure legend
                chart
                    .configure_series_labels()
                    .background_style(&WHITE.mix(0.8))
                    .border_style(&BLACK)
                    .draw()
                    .map_err(|e| format!("Legend drawing error: {}", e))?;
            }

            Ok(String::from_utf8(buffer).map_err(|e| format!("UTF-8 conversion error: {}", e))?)
        }

        #[cfg(not(feature = "visualization"))]
        {
            Err(
                "Visualization feature not enabled. Compile with --features visualization"
                    .to_string(),
            )
        }
    }

    pub fn generate_level_distribution_chart(
        &self,
        _data: &VisualizationData,
    ) -> Result<String, String> {
        #[cfg(feature = "visualization")]
        {
            use plotters::prelude::*;

            let mut buffer = Vec::new();
            {
                let root =
                    SVGBackend::with_buffer(&mut buffer, (self.config.width, self.config.height))
                        .into_drawing_area();

                root.fill(&WHITE)
                    .map_err(|e| format!("Drawing error: {}", e))?;

                let total: usize = data.level_distribution.values().sum();
                let levels: Vec<_> = data.level_distribution.iter().collect();

                let mut chart = ChartBuilder::on(&root)
                    .caption("Log Level Distribution", ("sans-serif", 20).into_font())
                    .margin(10)
                    .x_label_area_size(40)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        0..levels.len(),
                        0..*levels.iter().map(|(_, &count)| count).max().unwrap_or(&1),
                    )
                    .map_err(|e| format!("Chart building error: {}", e))?;

                // Configure mesh
                chart
                    .configure_mesh()
                    .x_desc("Log Level")
                    .y_desc("Count")
                    .x_label_style(("sans-serif", 12))
                    .draw()
                    .map_err(|e| format!("Mesh drawing error: {}", e))?;

                // Draw bars
                for (i, (level, &count)) in levels.iter().enumerate() {
                    let color = self.config.colors[i % self.config.colors.len()]
                        .parse::<RGBColor>()
                        .unwrap_or(BLUE);

                    chart
                        .draw_series(Rectangle::new(
                            [(i as f32 - 0.4, 0), (i as f32 + 0.4, count as f32)],
                            color.filled(),
                        ))
                        .map_err(|e| format!("Bar drawing error: {}", e))?
                        .label(format!(
                            "{} ({:.1}%)",
                            level,
                            (count as f64 / total as f64) * 100.0
                        ));
                }

                // Configure legend
                chart
                    .configure_series_labels()
                    .background_style(&WHITE.mix(0.8))
                    .border_style(&BLACK)
                    .draw()
                    .map_err(|e| format!("Legend drawing error: {}", e))?;
            }

            Ok(String::from_utf8(buffer).map_err(|e| format!("UTF-8 conversion error: {}", e))?)
        }

        #[cfg(not(feature = "visualization"))]
        {
            Err(
                "Visualization feature not enabled. Compile with --features visualization"
                    .to_string(),
            )
        }
    }

    pub fn generate_pattern_heatmap(&self, _data: &VisualizationData) -> Result<String, String> {
        #[cfg(feature = "visualization")]
        {
            use plotters::prelude::*;

            let mut buffer = Vec::new();
            {
                let root =
                    SVGBackend::with_buffer(&mut buffer, (self.config.width, self.config.height))
                        .into_drawing_area();

                root.fill(&WHITE)
                    .map_err(|e| format!("Drawing error: {}", e))?;

                let max_freq = data.patterns.first().map(|p| p.frequency).unwrap_or(1);

                let mut chart = ChartBuilder::on(&root)
                    .caption("Pattern Frequency Heatmap", ("sans-serif", 20).into_font())
                    .margin(10)
                    .x_label_area_size(40)
                    .y_label_area_size(40)
                    .build_cartesian_2d(0..data.patterns.len().max(10), 0..max_freq)
                    .map_err(|e| format!("Chart building error: {}", e))?;

                // Configure mesh
                chart
                    .configure_mesh()
                    .x_desc("Pattern Index")
                    .y_desc("Frequency")
                    .draw()
                    .map_err(|e| format!("Mesh drawing error: {}", e))?;

                // Draw heatmap rectangles
                for (i, pattern) in data.patterns.iter().take(10).enumerate() {
                    let intensity = pattern.frequency as f64 / max_freq as f64;
                    let color = self.get_heatmap_color(intensity);

                    chart
                        .draw_series(Rectangle::new(
                            [
                                (i as f32 - 0.4, 0),
                                (i as f32 + 0.4, pattern.frequency as f32),
                            ],
                            color.filled(),
                        ))
                        .map_err(|e| format!("Heatmap drawing error: {}", e))?;
                }

                // Add pattern labels
                for (i, pattern) in data.patterns.iter().take(5).enumerate() {
                    chart
                        .draw_series(std::iter::once(Text::new(
                            (i as f32, pattern.frequency as f32 + 1),
                            format!("P{}", i + 1),
                            ("sans-serif", 10).into_font(),
                        )))
                        .map_err(|e| format!("Text drawing error: {}", e))?;
                }
            }

            Ok(String::from_utf8(buffer).map_err(|e| format!("UTF-8 conversion error: {}", e))?)
        }

        #[cfg(not(feature = "visualization"))]
        {
            Err(
                "Visualization feature not enabled. Compile with --features visualization"
                    .to_string(),
            )
        }
    }

    #[cfg(feature = "visualization")]
    #[allow(dead_code)]
    fn get_heatmap_color(&self, intensity: f64) -> RGBColor {
        // Color gradient from blue (low) to red (high)
        if intensity < 0.25 {
            RGBColor(0, 0, 255) // Blue
        } else if intensity < 0.5 {
            RGBColor(0, 255, 255) // Cyan
        } else if intensity < 0.75 {
            RGBColor(0, 255, 0) // Green
        } else if intensity < 0.9 {
            RGBColor(255, 255, 0) // Yellow
        } else {
            RGBColor(255, 0, 0) // Red
        }
    }

    #[cfg(not(feature = "visualization"))]
    #[allow(dead_code)]
    fn get_heatmap_color(&self, _intensity: f64) -> String {
        // Return a simple color name when visualization is not available
        "blue".to_string()
    }

    pub fn print_text_summary(&self, data: &VisualizationData) {
        println!("=== Log Analysis Summary ===");

        // Timeline summary
        if let (Some(first), Some(last)) = (data.timeline.first(), data.timeline.last()) {
            println!("Time Range: {} to {}", first.timestamp, last.timestamp);
            println!(
                "Total Entries: {}",
                data.timeline.iter().map(|p| p.count).sum::<usize>()
            );
            println!(
                "Peak Rate: {} entries/minute",
                data.timeline.iter().map(|p| p.count).max().unwrap_or(0)
            );
        }

        // Level distribution
        println!("\n--- Level Distribution ---");
        let total: usize = data.level_distribution.values().sum();
        for (level, count) in &data.level_distribution {
            let percentage = (*count as f64 / total as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", level, count, percentage);
        }

        // Top patterns
        println!("\n--- Top Patterns ---");
        for (i, pattern) in data.patterns.iter().take(5).enumerate() {
            println!(
                "  {}. {} ({} times)",
                i + 1,
                pattern.pattern,
                pattern.frequency
            );
            if !pattern.examples.is_empty() {
                println!("     Example: {}", pattern.examples[0]);
            }
        }

        // Anomalies
        if !data.anomalies.is_empty() {
            println!("\n--- Anomalies Detected ---");
            for (i, anomaly) in data.anomalies.iter().take(3).enumerate() {
                println!("  {}. Score: {:.2}", i + 1, anomaly.score);
                println!("     {}", anomaly.message);
                println!("     Time: {}", anomaly.entry.timestamp);
            }
        }

        // Recommendations
        println!("\n--- Recommendations ---");
        self.generate_recommendations(data);
    }

    fn generate_recommendations(&self, data: &VisualizationData) {
        let error_count = data.level_distribution.get("ERROR").unwrap_or(&0);
        let warn_count = data.level_distribution.get("WARN").unwrap_or(&0);
        let total: usize = data.level_distribution.values().sum();

        if *error_count as f64 / total as f64 > 0.1 {
            println!(
                "  ⚠️  High error rate detected ({:.1}%). Consider investigating error patterns.",
                (*error_count as f64 / total as f64) * 100.0
            );
        }

        if *warn_count as f64 / total as f64 > 0.2 {
            println!(
                "  ⚠️  High warning rate detected ({:.1}%). Monitor for potential issues.",
                (*warn_count as f64 / total as f64) * 100.0
            );
        }

        if !data.anomalies.is_empty() {
            println!(
                "  🔍 {} anomalies detected. Review unusual log patterns.",
                data.anomalies.len()
            );
        }

        if let Some(peak) = data.timeline.iter().map(|p| p.count).max() {
            let avg = data.timeline.iter().map(|p| p.count).sum::<usize>() as f64
                / data.timeline.len() as f64;
            if peak as f64 > avg * 3.0 {
                println!("  📈 High volume spikes detected. Consider load balancing or scaling.");
            }
        }

        if data.patterns.len() > 20 {
            println!("  🔄 Many unique patterns detected. Consider log standardization.");
        }
    }

    pub fn export_json(&self, data: &VisualizationData) -> Result<String, String> {
        serde_json::to_string_pretty(data).map_err(|e| format!("JSON serialization error: {}", e))
    }

    pub fn export_csv(&self, data: &VisualizationData) -> Result<String, String> {
        let mut csv = String::new();

        // Header
        csv.push_str("timestamp,count,level,level_count\n");

        // Timeline data
        for point in &data.timeline {
            for (level, count) in &point.level_breakdown {
                csv.push_str(&format!(
                    "{},{},{},{}\n",
                    point.timestamp, point.count, level, count
                ));
            }
        }

        Ok(csv)
    }
}

impl Default for LogVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::VisualizationData;
    use std::collections::HashMap;

    #[test]
    fn test_text_summary() {
        let visualizer = LogVisualizer::new();
        let data = VisualizationData {
            timeline: vec![],
            level_distribution: vec![("INFO".to_string(), 10)].into_iter().collect(),
            patterns: vec![],
            anomalies: vec![],
        };

        // This should not panic
        visualizer.print_text_summary(&data);
    }

    #[test]
    fn test_json_export() {
        let visualizer = LogVisualizer::new();
        let data = VisualizationData {
            timeline: vec![],
            level_distribution: HashMap::new(),
            patterns: vec![],
            anomalies: vec![],
        };

        let json = visualizer.export_json(&data).unwrap();
        assert!(json.contains("timeline"));
        assert!(json.contains("level_distribution"));
    }

    #[test]
    fn test_csv_export() {
        let visualizer = LogVisualizer::new();
        let data = VisualizationData {
            timeline: vec![],
            level_distribution: HashMap::new(),
            patterns: vec![],
            anomalies: vec![],
        };

        let csv = visualizer.export_csv(&data).unwrap();
        assert!(csv.contains("timestamp,count,level,level_count"));
    }
}
