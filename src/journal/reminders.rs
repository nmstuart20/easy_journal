use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::error::{JournalError, Result};

const APPLESCRIPT_GET_REMINDERS: &str = r#"
tell application "Reminders"
    -- 1. Fetch all names in a single IPC call using a 'whose' filter
    -- This returns a list of lists: {{"Task A", "Task B"}, {"Task C"}, {}}
    set nestedNames to name of (reminders of every list whose completed is false)
    
    -- 2. Flatten and join the lists using AppleScript's text delimiters
    set {oldTID, AppleScript's text item delimiters} to {AppleScript's text item delimiters, linefeed}
    set output to nestedNames as text
    set AppleScript's text item delimiters to oldTID
    
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

    // Spawn the process instead of using .output() to allow for timeout
    let mut child = Command::new("osascript")
        .arg("-") // Read script from stdin
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            JournalError::RemindersFailed(format!("Failed to execute AppleScript: {}", e))
        })?;

    // Write the AppleScript to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(APPLESCRIPT_GET_REMINDERS.as_bytes())
            .map_err(|e| {
                JournalError::RemindersFailed(format!(
                    "Failed to write AppleScript to stdin: {}",
                    e
                ))
            })?;
    }

    // Wait for the process with a timeout (5 seconds)
    let timeout = Duration::from_secs(120);
    let start = std::time::Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has finished
                let output = child.wait_with_output().map_err(|e| {
                    JournalError::RemindersFailed(format!("Failed to read output: {}", e))
                })?;

                if !status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(JournalError::RemindersFailed(format!(
                        "AppleScript execution failed: {}",
                        stderr
                    )));
                }

                let stdout = String::from_utf8(output.stdout).map_err(|e| {
                    JournalError::RemindersFailed(format!("Invalid UTF-8 in output: {}", e))
                })?;

                let reminders: Vec<String> = stdout
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect();

                return Ok(reminders);
            }
            Ok(None) => {
                // Process is still running
                if start.elapsed() > timeout {
                    // Timeout exceeded, kill the process
                    let _ = child.kill();
                    return Err(JournalError::RemindersFailed(
                        "AppleScript execution timed out after 5 seconds. \
                        Please check System Settings > Privacy & Security > Automation \
                        and ensure your terminal has permission to access Reminders."
                            .to_string(),
                    ));
                }
                // Sleep briefly before checking again
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(JournalError::RemindersFailed(format!(
                    "Error waiting for AppleScript: {}",
                    e
                )));
            }
        }
    }
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

    #[test]
    #[cfg(target_os = "macos")]
    fn test_fetch_reminders_access() {
        // This test verifies that the system can access Apple Reminders
        // It will fail with a helpful message if permissions are not granted
        match fetch_reminders_applescript() {
            Ok(reminders) => {
                println!("Successfully fetched {} reminders", reminders.len());
                // Success - we have permission to access Reminders
            }
            Err(e) => {
                let error_msg = format!("{}", e);

                if error_msg.contains("timed out") {
                    panic!(
                        "Failed to access Apple Reminders - permission likely not granted.\n\
                        Please check System Settings > Privacy & Security > Automation\n\
                        and ensure your terminal has permission to access Reminders.\n\
                        Error: {}",
                        error_msg
                    );
                } else {
                    // Other errors (like Reminders app not available) are also failures
                    panic!("Failed to fetch reminders: {}", error_msg);
                }
            }
        }
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_fetch_reminders_non_macos() {
        // On non-macOS systems, should return empty vector
        let result = fetch_reminders_applescript();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<String>::new());
    }
}
