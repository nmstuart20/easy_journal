use std::env;
use std::process::Command;

use crate::error::{JournalError, Result};

const APPLESCRIPT_GET_REMINDERS: &str = r#"
tell application "Reminders"
    set output to ""
    set allLists to lists
    repeat with aList in allLists
        set listReminders to (reminders of aList whose completed is false)
        repeat with aReminder in listReminders
            set output to output & (name of aReminder) & linefeed
        end repeat
    end repeat
    return output
end tell
"#;

fn is_macos() -> bool {
    env::consts::OS == "macos"
}

fn fetch_reminders_applescript() -> Result<Vec<String>> {
    if !is_macos() {
        return Ok(Vec::new());
    }

    let output = Command::new("osascript")
        .arg("-e")
        .arg(APPLESCRIPT_GET_REMINDERS)
        .output()
        .map_err(|e| {
            JournalError::RemindersFailed(format!("Failed to execute AppleScript: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JournalError::RemindersFailed(format!(
            "AppleScript execution failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| JournalError::RemindersFailed(format!("Invalid UTF-8 in output: {}", e)))?;

    let reminders: Vec<String> = stdout
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    Ok(reminders)
}

fn format_reminders(reminders: Vec<String>) -> String {
    reminders
        .iter()
        .map(|reminder| format!("- [ ] {}", reminder))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn fetch_apple_reminders() -> Result<Option<String>> {
    match fetch_reminders_applescript() {
        Ok(reminders) => {
            if reminders.is_empty() {
                Ok(None)
            } else {
                Ok(Some(format_reminders(reminders)))
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not fetch Apple Reminders: {}", e);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_reminders() {
        let reminders = vec![
            "Buy groceries".to_string(),
            "Call dentist".to_string(),
            "Review PR".to_string(),
        ];

        let formatted = format_reminders(reminders);

        assert_eq!(
            formatted,
            "- [ ] Buy groceries\n- [ ] Call dentist\n- [ ] Review PR"
        );
    }

    #[test]
    fn test_format_empty_reminders() {
        let reminders: Vec<String> = vec![];
        let formatted = format_reminders(reminders);
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_is_macos() {
        let result = is_macos();
        assert_eq!(result, cfg!(target_os = "macos"));
    }
}
