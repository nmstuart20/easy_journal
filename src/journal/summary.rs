use chrono::NaiveDate;
use std::fs;
use std::path::Path;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
enum SummaryNode {
    UserContent(String),
    Separator,
    YearHeader(u32),
    MonthEntry {
        year: u32,
        month: u32,
        month_name: String,
    },
    DayEntry {
        year: u32,
        month: u32,
        day: u32,
        day_of_week: String,
    },
}

pub struct Summary {
    nodes: Vec<SummaryNode>,
    path: std::path::PathBuf,
}

impl Summary {
    pub fn parse(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut nodes = Vec::new();
        let mut in_user_content = true;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for separator
            if trimmed == "---" {
                nodes.push(SummaryNode::Separator);
                in_user_content = false;
                continue;
            }

            // If we're still in user content area
            if in_user_content {
                nodes.push(SummaryNode::UserContent(line.to_string()));
                continue;
            }

            // Parse year headers - both plain and linked formats
            // "# 2025" or "# [2025](2025/README.md)"
            if let Some(year_str) = trimmed.strip_prefix("# ") {
                // Handle linked format: # [2025](2025/README.md)
                if let Some((year_label, _)) = parse_year_entry(year_str)
                    && let Ok(year) = year_label.parse::<u32>()
                {
                    nodes.push(SummaryNode::YearHeader(year));
                    continue;
                }
                // Handle plain format: # 2025
                if let Ok(year) = year_str.parse::<u32>() {
                    nodes.push(SummaryNode::YearHeader(year));
                    continue;
                }
            }

            // Parse day entries (e.g., "  - [29 - Sunday](2025/12/29.md)")
            // Check original line for indentation, not trimmed
            if line.starts_with("  - [")
                && let Some((day_label, path)) = parse_day_entry(trimmed)
                && let Some((year, month, day, day_of_week)) =
                    extract_day_info_from_path(&path, &day_label)
            {
                nodes.push(SummaryNode::DayEntry {
                    year,
                    month,
                    day,
                    day_of_week,
                });
                continue;
            }

            // Parse month entries (e.g., "- [December](2025/12/README.md)")
            // Must come after day entries check
            if trimmed.starts_with("- [")
                && let Some((month_name, path)) = parse_month_entry(trimmed)
                && let Some((year, month)) = extract_year_month_from_path(&path)
            {
                nodes.push(SummaryNode::MonthEntry {
                    year,
                    month,
                    month_name,
                });
                continue;
            }

            // Skip empty lines and other content after separator
            if trimmed.is_empty() || !in_user_content {
                continue;
            }

            // Preserve other user content
            nodes.push(SummaryNode::UserContent(line.to_string()));
        }

        Ok(Summary {
            nodes,
            path: path.to_path_buf(),
        })
    }

    pub fn add_day_entry(&mut self, date: NaiveDate) {
        let year = date.format("%Y").to_string().parse::<u32>().unwrap();
        let month = date.format("%m").to_string().parse::<u32>().unwrap();
        let day = date.format("%d").to_string().parse::<u32>().unwrap();
        let day_of_week = date.format("%A").to_string();

        // Check if entry already exists
        for node in &self.nodes {
            if let SummaryNode::DayEntry {
                year: y,
                month: m,
                day: d,
                ..
            } = node
                && *y == year
                && *m == month
                && *d == day
            {
                return; // Entry already exists
            }
        }

        // Ensure separator exists
        if !self
            .nodes
            .iter()
            .any(|n| matches!(n, SummaryNode::Separator))
        {
            self.nodes.push(SummaryNode::Separator);
        }

        // Find or create year header
        let year_idx = self.find_or_insert_year(year);

        // Find or create month entry
        let month_name = get_month_name(month);
        self.find_or_insert_month(year, month, month_name, year_idx);

        // Insert day entry
        self.insert_day(year, month, day, day_of_week);
    }

    fn find_or_insert_year(&mut self, year: u32) -> usize {
        // Find the separator first
        let sep_idx = self
            .nodes
            .iter()
            .position(|n| matches!(n, SummaryNode::Separator))
            .unwrap();

        // Look for existing year
        for (i, node) in self.nodes.iter().enumerate().skip(sep_idx) {
            if let SummaryNode::YearHeader(y) = node {
                if *y == year {
                    return i;
                }
                if *y < year {
                    // Insert new year before this one (reverse chronological)
                    self.nodes.insert(i, SummaryNode::YearHeader(year));
                    self.nodes
                        .insert(i + 1, SummaryNode::UserContent(String::new()));
                    return i;
                }
            }
        }

        // No year found or all years are newer, append at the end
        self.nodes.push(SummaryNode::UserContent(String::new()));
        self.nodes.push(SummaryNode::YearHeader(year));
        self.nodes.len() - 1
    }

    fn find_or_insert_month(&mut self, year: u32, month: u32, month_name: String, year_idx: usize) {
        // Look for existing month under this year
        let mut insert_pos = None;
        let mut found = false;

        for (i, node) in self.nodes.iter().enumerate().skip(year_idx + 1) {
            match node {
                SummaryNode::YearHeader(_) => {
                    // Reached next year, insert before it
                    insert_pos = Some(i);
                    break;
                }
                SummaryNode::MonthEntry {
                    year: y,
                    month: m,
                    month_name: _,
                } if *y == year => {
                    if *m == month {
                        found = true;
                        break;
                    }
                    if *m < month {
                        // Insert before this month (reverse chronological)
                        insert_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        if !found {
            let pos = insert_pos.unwrap_or(self.nodes.len());
            self.nodes.insert(
                pos,
                SummaryNode::MonthEntry {
                    year,
                    month,
                    month_name,
                },
            );
        }
    }

    fn insert_day(&mut self, year: u32, month: u32, day: u32, day_of_week: String) {
        // Find the month entry
        let month_idx = self
            .nodes
            .iter()
            .position(|n| {
                matches!(
                    n,
                    SummaryNode::MonthEntry {
                        year: y,
                        month: m,
                        ..
                    } if *y == year && *m == month
                )
            })
            .unwrap();

        // Find where to insert the day (reverse chronological)
        let mut insert_pos = None;

        for (i, node) in self.nodes.iter().enumerate().skip(month_idx + 1) {
            match node {
                SummaryNode::MonthEntry { .. } | SummaryNode::YearHeader(_) => {
                    // Reached next month or year, insert before it
                    insert_pos = Some(i);
                    break;
                }
                SummaryNode::DayEntry {
                    year: y,
                    month: m,
                    day: d,
                    ..
                } if *y == year && *m == month => {
                    if *d < day {
                        // Insert before this day (reverse chronological)
                        insert_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let pos = insert_pos.unwrap_or(self.nodes.len());
        self.nodes.insert(
            pos,
            SummaryNode::DayEntry {
                year,
                month,
                day,
                day_of_week,
            },
        );
    }

    pub fn write(&self) -> Result<()> {
        let mut content = String::new();
        let mut in_user_content = true;

        for node in &self.nodes {
            match node {
                SummaryNode::UserContent(line) => {
                    content.push_str(line);
                    content.push('\n');
                }
                SummaryNode::Separator => {
                    content.push_str("---\n");
                    in_user_content = false;
                }
                SummaryNode::YearHeader(year) => {
                    if !in_user_content {
                        content.push('\n');
                    }
                    // Render as clickable link to year README
                    content.push_str(&format!("# [{}]({}/README.md)\n", year, year));
                }
                SummaryNode::MonthEntry {
                    year,
                    month,
                    month_name,
                } => {
                    content.push_str(&format!(
                        "- [{}]({}/{:02}/README.md)\n",
                        month_name, year, month
                    ));
                }
                SummaryNode::DayEntry {
                    year,
                    month,
                    day,
                    day_of_week,
                } => {
                    content.push_str(&format!(
                        "  - [{:02} - {}]({}/{:02}/{:02}.md)\n",
                        day, day_of_week, year, month, day
                    ));
                }
            }
        }

        fs::write(&self.path, content)?;
        Ok(())
    }
}

fn parse_month_entry(line: &str) -> Option<(String, String)> {
    // Parse "- [December](2025/12/README.md)"
    let line = line.trim_start_matches("- [");
    let parts: Vec<&str> = line.split("](").collect();
    if parts.len() == 2 {
        let month_name = parts[0].to_string();
        let path = parts[1].trim_end_matches(')').to_string();
        Some((month_name, path))
    } else {
        None
    }
}

fn parse_day_entry(line: &str) -> Option<(String, String)> {
    // Parse "  - [29 - Sunday](2025/12/29.md)"
    let line = line.trim_start_matches("  - [");
    let parts: Vec<&str> = line.split("](").collect();
    if parts.len() == 2 {
        let day_label = parts[0].to_string();
        let path = parts[1].trim_end_matches(')').to_string();
        Some((day_label, path))
    } else {
        None
    }
}

fn parse_year_entry(line: &str) -> Option<(String, String)> {
    // Parse "[2025](2025/README.md)"
    if line.starts_with('[') {
        let line = line.trim_start_matches('[');
        let parts: Vec<&str> = line.split("](").collect();
        if parts.len() == 2 {
            let year_label = parts[0].to_string();
            let path = parts[1].trim_end_matches(')').to_string();
            return Some((year_label, path));
        }
    }
    None
}

fn extract_year_month_from_path(path: &str) -> Option<(u32, u32)> {
    // Parse "2025/12/README.md" -> (2025, 12)
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        let year = parts[0].parse::<u32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        Some((year, month))
    } else {
        None
    }
}

fn extract_day_info_from_path(path: &str, label: &str) -> Option<(u32, u32, u32, String)> {
    // Parse path "2025/12/29.md" -> (2025, 12, 29)
    // Parse label "29 - Sunday" -> day_of_week
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 3 {
        let year = parts[0].parse::<u32>().ok()?;
        let month = parts[1].parse::<u32>().ok()?;
        let day_str = parts[2].trim_end_matches(".md");
        let day = day_str.parse::<u32>().ok()?;

        let day_of_week = label.split(" - ").nth(1).unwrap_or("Unknown").to_string();

        Some((year, month, day, day_of_week))
    } else {
        None
    }
}

fn get_month_name(month: u32) -> String {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
    .to_string()
}
