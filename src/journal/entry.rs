use chrono::{Duration, NaiveDate};
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::Result;
use crate::journal::{filesystem, parser, reminders, summary, template};

pub struct JournalEntry {
    pub date: NaiveDate,
    pub file_path: PathBuf,
}

impl JournalEntry {
    pub fn create(date: NaiveDate, config: &Config) -> Result<Self> {
        let year = date.format("%Y").to_string().parse::<u32>().unwrap();
        let month = date.format("%m").to_string().parse::<u32>().unwrap();

        // Ensure year and month directories exist
        filesystem::ensure_year_dir(year, &config.journal_dir)?;
        filesystem::ensure_month_dir(year, month, &config.journal_dir)?;
        filesystem::create_month_readme(year, month, &config.journal_dir)?;

        // Get entry path
        let entry_path = filesystem::get_entry_path(date, &config.journal_dir);

        // Create entry file if it doesn't exist
        if !entry_path.exists() {
            let template_content = template::load_template(&config.template_path)?;

            // Get previous entry's unchecked tasks and "Tomorrow's Focus" content
            let previous_content = Self::get_previous_content(date, config)?;

            // Fetch Apple Reminders
            let apple_reminders = reminders::fetch_apple_reminders().unwrap_or(None);

            let content = template::apply_variables(
                &template_content,
                date,
                previous_content,
                apple_reminders,
            );
            fs::write(&entry_path, content)?;

            // Update SUMMARY.md
            let summary_path = config.journal_dir.join("SUMMARY.md");
            let mut summary = summary::Summary::parse(&summary_path)?;
            summary.add_day_entry(date);
            summary.write()?;
        }

        Ok(JournalEntry {
            date,
            file_path: entry_path,
        })
    }

    pub fn exists(date: NaiveDate, config: &Config) -> bool {
        let entry_path = filesystem::get_entry_path(date, &config.journal_dir);
        entry_path.exists()
    }

    /// Find the most recent entry before the given date (within 30 days)
    pub fn find_previous_entry(date: NaiveDate, config: &Config) -> Option<PathBuf> {
        // Search backwards up to 30 days
        for days_back in 1..=30 {
            if let Some(prev_date) = date.checked_sub_signed(Duration::days(days_back)) {
                let entry_path = filesystem::get_entry_path(prev_date, &config.journal_dir);
                if entry_path.exists() {
                    return Some(entry_path);
                }
            }
        }
        None
    }

    /// Get unchecked tasks and "Tomorrow's Focus" content from the previous entry
    pub fn get_previous_content(date: NaiveDate, config: &Config) -> Result<Option<String>> {
        if let Some(prev_entry_path) = Self::find_previous_entry(date, config) {
            let content = fs::read_to_string(&prev_entry_path)?;

            // Extract unchecked tasks from "Goals for Today"
            let unchecked_tasks = parser::extract_unchecked_tasks(&content);

            // Extract "Tomorrow's Focus" section
            let tomorrow_focus = parser::extract_section(&content, "Tomorrow's Focus");

            // Combine both: unchecked tasks first, then tomorrow's focus
            match (unchecked_tasks, tomorrow_focus) {
                (Some(tasks), Some(focus)) => Ok(Some(format!("{}\n{}", tasks, focus))),
                (Some(tasks), None) => Ok(Some(tasks)),
                (None, Some(focus)) => Ok(Some(focus)),
                (None, None) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}
