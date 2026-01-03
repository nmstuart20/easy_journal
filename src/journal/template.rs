use chrono::NaiveDate;
use std::fs;
use std::path::Path;

use crate::error::{JournalError, Result};

const DEFAULT_TEMPLATE: &str = r#"# {{date}} - {{day_of_week}}

## Reminders
{{reminders}}

## Goals for Today
- [ ]
- [ ]
- [ ]

## Work Accomplished

### Morning
-

### Afternoon
-

### Evening
-

## Learning & Insights
-

## Challenges & Blockers
-

## Gratitude & Wins
-

## Tomorrow's Focus
-

---

**Mood**:
**Energy Level**:
**Hours Worked**:
"#;

pub fn load_template(template_path: &Path) -> Result<String> {
    if template_path.exists() {
        fs::read_to_string(template_path).map_err(JournalError::Io)
    } else {
        Ok(DEFAULT_TEMPLATE.to_string())
    }
}

pub fn apply_variables(
    template: &str,
    date: NaiveDate,
    previous_content: Option<String>,
    reminders: Option<String>,
) -> String {
    let date_str = date.format("%Y-%m-%d").to_string();
    let day_of_week = date.format("%A").to_string();
    let year = date.format("%Y").to_string();
    let month = date.format("%B").to_string();
    let month_num = date.format("%m").to_string();
    let day = date.format("%d").to_string();

    let reminders_content = reminders.unwrap_or_default();
    
    let mut result = template
        .replace("{{date}}", &date_str)
        .replace("{{day_of_week}}", &day_of_week)
        .replace("{{year}}", &year)
        .replace("{{month}}", &month)
        .replace("{{month_num}}", &month_num)
        .replace("{{day}}", &day)
        .replace("{{reminders}}", &reminders_content);

    // If we have previous content, inject it into "Goals for Today"
    if let Some(content) = previous_content {
        result = inject_previous_content(&result, &content);
    }

    result
}

/// Inject previous content (unfinished tasks and tomorrow's focus) into the "Goals for Today" section
fn inject_previous_content(template: &str, content: &str) -> String {
    let lines: Vec<&str> = template.lines().collect();
    let mut result = String::new();
    let mut in_goals_section = false;
    let mut added_content = false;

    for line in lines {
        let trimmed = line.trim();

        // Check if we're starting the "Goals for Today" section
        if trimmed.starts_with("##") && trimmed.contains("Goals for Today") {
            result.push_str(line);
            result.push('\n');
            in_goals_section = true;
            continue;
        }

        // If we're in the goals section and haven't added content yet
        if in_goals_section && !added_content {
            // Check if we've hit another section or separator
            if trimmed.starts_with("##") || trimmed.starts_with("---") {
                // Convert content to checkboxes and add before this line
                let checkbox_content = convert_to_checkboxes(content);
                result.push_str(&checkbox_content);
                result.push('\n');
                result.push('\n');
                added_content = true;
                in_goals_section = false;
            }
            // Skip placeholder lines in the goals section
            else if !trimmed.is_empty() && (trimmed == "- [ ]" || trimmed == "-") {
                continue;
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Convert list items to checkbox format
fn convert_to_checkboxes(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            // If it's already a checkbox, keep it
            if trimmed.starts_with("- [ ]")
                || trimmed.starts_with("- [x]")
                || trimmed.starts_with("- [X]")
            {
                line.to_string()
            }
            // If it's a regular list item, convert to checkbox
            else if trimmed.starts_with("- ") {
                line.replacen("- ", "- [ ] ", 1)
            }
            // Otherwise, keep as is
            else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_apply_variables() {
        let template = "# {{date}} - {{day_of_week}}\nYear: {{year}}, Month: {{month}}";
        let date = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap();
        let result = apply_variables(template, date, None, None);

        assert!(result.contains("2025-12-29"));
        assert!(result.contains("Monday"));
        assert!(result.contains("Year: 2025"));
        assert!(result.contains("Month: December"));
    }

    #[test]
    fn test_inject_previous_content() {
        let template = r#"# 2025-12-30

## Goals for Today
- [ ]
- [ ]

## Work Accomplished
-"#;

        let previous_content = "- Complete feature X\n- Review documentation";
        let result = inject_previous_content(template, previous_content);

        assert!(result.contains("- [ ] Complete feature X"));
        assert!(result.contains("- [ ] Review documentation"));
    }

    #[test]
    fn test_reminders_variable() {
        let template = "## Reminders\n{{reminders}}\n## Goals";
        let date = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap();
        let reminders = Some("- [ ] Buy milk\n- [ ] Call dentist".to_string());
        let result = apply_variables(template, date, None, reminders);

        assert!(result.contains("- [ ] Buy milk"));
        assert!(result.contains("- [ ] Call dentist"));
        assert!(!result.contains("{{reminders}}"));
    }

    #[test]
    fn test_convert_to_checkboxes() {
        let content = "- Task 1\n- Task 2\n- [ ] Already a checkbox";
        let result = convert_to_checkboxes(content);

        assert!(result.contains("- [ ] Task 1"));
        assert!(result.contains("- [ ] Task 2"));
        assert!(result.contains("- [ ] Already a checkbox"));
        // Make sure we didn't double-add checkboxes
        assert!(!result.contains("- [ ] - [ ]"));
    }
}
