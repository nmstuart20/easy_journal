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

const DEFAULT_MONTH_TEMPLATE: &str = r#"# {{month}} {{year}}

## Goals for this month
- [ ]
- [ ]
- [ ]

## Key Projects & Focus Areas

### Project 1


### Project 2


## Reflections & Learnings


## Highlights & Accomplishments


---

**Month Started**:
**Month Rating (1-10)**:
"#;

const DEFAULT_YEAR_TEMPLATE: &str = r#"# Year in Review: {{year}}

## Goals for the Year

### Professional Goals
- [ ]
- [ ]
- [ ]

### Personal Goals
- [ ]
- [ ]
- [ ]

### Health & Wellness Goals
- [ ]
- [ ]

## Themes or Focus Areas

### Theme 1:


### Theme 2:


### Theme 3:


## Highlights & Accomplishments

### Q1 (Jan-Mar)


### Q2 (Apr-Jun)


### Q3 (Jul-Sep)


### Q4 (Oct-Dec)


## Challenges & Growth


## Lessons Learned


---

**Year Started**:
**Overall Year Rating (1-10)**:
"#;

pub fn load_template(template_path: &Path) -> Result<String> {
    if template_path.exists() {
        fs::read_to_string(template_path).map_err(JournalError::Io)
    } else {
        Ok(DEFAULT_TEMPLATE.to_string())
    }
}

pub fn load_month_template(template_path: &Path) -> Result<String> {
    if template_path.exists() {
        fs::read_to_string(template_path).map_err(JournalError::Io)
    } else {
        Ok(DEFAULT_MONTH_TEMPLATE.to_string())
    }
}

pub fn load_year_template(template_path: &Path) -> Result<String> {
    if template_path.exists() {
        fs::read_to_string(template_path).map_err(JournalError::Io)
    } else {
        Ok(DEFAULT_YEAR_TEMPLATE.to_string())
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

pub fn apply_month_variables(template: &str, year: u32, month: u32) -> String {
    let month_name = get_month_name(month);
    let date_str = format!("{}-{:02}", year, month);

    template
        .replace("{{year}}", &year.to_string())
        .replace("{{month}}", month_name)
        .replace("{{month_num}}", &format!("{:02}", month))
        .replace("{{date}}", &date_str)
}

pub fn apply_year_variables(template: &str, year: u32) -> String {
    template
        .replace("{{year}}", &year.to_string())
        .replace("{{date}}", &year.to_string())
}

fn get_month_name(month: u32) -> &'static str {
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

    #[test]
    fn test_apply_month_variables() {
        let template = "# {{month}} {{year}} - {{month_num}}\n{{date}}";
        let result = apply_month_variables(template, 2025, 12);
        assert!(result.contains("December"));
        assert!(result.contains("2025"));
        assert!(result.contains("12"));
        assert!(result.contains("2025-12"));
    }

    #[test]
    fn test_apply_year_variables() {
        let template = "# Year {{year}} Review\n{{date}}";
        let result = apply_year_variables(template, 2025);
        assert!(result.contains("2025"));
        assert_eq!(result.lines().count(), 2);
    }

    #[test]
    fn test_get_month_name() {
        assert_eq!(get_month_name(1), "January");
        assert_eq!(get_month_name(6), "June");
        assert_eq!(get_month_name(12), "December");
        assert_eq!(get_month_name(13), "Unknown");
    }

    #[test]
    fn test_load_month_template_default() {
        // Test that default template is returned when file doesn't exist
        let result = load_month_template(Path::new("nonexistent_month_template.md")).unwrap();
        assert!(result.contains("{{month}} {{year}}"));
        assert!(result.contains("Goals for this month"));
    }

    #[test]
    fn test_load_year_template_default() {
        // Test that default template is returned when file doesn't exist
        let result = load_year_template(Path::new("nonexistent_year_template.md")).unwrap();
        assert!(result.contains("Year in Review: {{year}}"));
        assert!(result.contains("Goals for the Year"));
    }
}
