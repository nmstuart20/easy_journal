use chrono::{Local, NaiveDate};
use std::env;
use std::process::Command;

use crate::config::Config;
use crate::error::{JournalError, Result};
use crate::journal::entry::JournalEntry;

pub async fn run(date_str: Option<String>, config: &Config) -> Result<()> {
    // Determine the date
    let date = if let Some(date_str) = date_str {
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|e| JournalError::DateParse(format!("Invalid date format: {}", e)))?
    } else {
        Local::now().date_naive()
    };

    // Create or get existing entry
    let entry = JournalEntry::create(date, config).await?;

    let exists_msg = if JournalEntry::exists(date, config) {
        "Opening existing entry"
    } else {
        "Created new entry"
    };

    println!("{} for {}", exists_msg, date.format("%Y-%m-%d"));
    println!(
        "Entry path: {:?} for entry date {:?}",
        entry.file_path, entry.date
    );

    // Open in editor
    open_in_editor(&entry.file_path.to_string_lossy())?;

    Ok(())
}

fn open_in_editor(path: &str) -> Result<()> {
    // Try to get editor from environment variables
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| {
            // Try common editors as fallback
            if Command::new("code").arg("--version").output().is_ok() {
                "code".to_string()
            } else if Command::new("vim").arg("--version").output().is_ok() {
                "vim".to_string()
            } else if Command::new("nano").arg("--version").output().is_ok() {
                "nano".to_string()
            } else if Command::new("vi").arg("--version").output().is_ok() {
                "vi".to_string()
            } else {
                "".to_string()
            }
        });

    if editor.is_empty() {
        println!("No editor found. Please set $EDITOR or $VISUAL environment variable.");
        println!("Entry created at: {}", path);
        return Ok(());
    }

    println!("Opening with editor: {}", editor);

    let status = Command::new(&editor)
        .arg(path)
        .status()
        .map_err(|e| JournalError::EditorFailed(format!("Failed to open editor: {}", e)))?;

    if !status.success() {
        return Err(JournalError::EditorFailed(format!(
            "Editor exited with status: {}",
            status
        )));
    }

    Ok(())
}
