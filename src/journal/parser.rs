/// Extract content from a markdown section
pub fn extract_section(content: &str, section_header: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut section_content = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Check if we're starting the target section
        if trimmed.starts_with("##") && trimmed.contains(section_header) {
            in_section = true;
            continue;
        }

        // Check if we've hit another section (stop collecting)
        if in_section && trimmed.starts_with("##") {
            break;
        }

        // Check if we've hit a horizontal rule (stop collecting)
        if in_section && trimmed.starts_with("---") {
            break;
        }

        // Collect content if we're in the section
        if in_section && !trimmed.is_empty() {
            section_content.push(line);
        }
    }

    if section_content.is_empty() {
        None
    } else {
        Some(section_content.join("\n").trim().to_string())
    }
}

/// Extract all unchecked tasks from the "Goals for Today" section
pub fn extract_unchecked_tasks(content: &str) -> Option<String> {
    // First, extract the "Goals for Today" section
    let goals_section = extract_section(content, "Goals for Today")?;

    // Filter for unchecked tasks only
    let unchecked: Vec<&str> = goals_section
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("- [ ]")
        })
        .collect();

    if unchecked.is_empty() {
        None
    } else {
        Some(unchecked.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_section() {
        let content = r#"# 2025-12-29 - Monday

## Goals for Today
- [ ] Task 1
- [ ] Task 2

## Tomorrow's Focus
- Work on feature X
- Review PR #123

## Notes
Some notes here
"#;

        let tomorrow = extract_section(content, "Tomorrow's Focus");
        assert!(tomorrow.is_some());
        let tomorrow = tomorrow.unwrap();
        assert!(tomorrow.contains("Work on feature X"));
        assert!(tomorrow.contains("Review PR #123"));

        let goals = extract_section(content, "Goals for Today");
        assert!(goals.is_some());
        assert!(goals.unwrap().contains("Task 1"));
    }

    #[test]
    fn test_extract_section_with_separator() {
        let content = r#"## Tomorrow's Focus
- Task A
- Task B

---

**Mood**: Good
"#;

        let tomorrow = extract_section(content, "Tomorrow's Focus");
        assert!(tomorrow.is_some());
        let tomorrow = tomorrow.unwrap();
        assert!(tomorrow.contains("Task A"));
        assert!(!tomorrow.contains("Mood")); // Should stop at ---
    }

    #[test]
    fn test_extract_unchecked_tasks() {
        let content = r#"# 2025-12-30 - Tuesday

## Goals for Today
- [ ] Take boxes to dump
- [x] Clean up leaves
- [ ] Leetcode
- [ ] CFP project work
- [x] Run miles

## Work Accomplished
- Did work
"#;

        let unchecked = extract_unchecked_tasks(content);
        assert!(unchecked.is_some());
        let tasks = unchecked.unwrap();
        assert!(tasks.contains("Take boxes to dump"));
        assert!(tasks.contains("Leetcode"));
        assert!(tasks.contains("CFP project work"));
        assert!(!tasks.contains("Clean up leaves"));
        assert!(!tasks.contains("Run miles"));
    }

    #[test]
    fn test_extract_unchecked_tasks_all_completed() {
        let content = r#"## Goals for Today
- [x] Task 1
- [x] Task 2
"#;

        let unchecked = extract_unchecked_tasks(content);
        assert!(unchecked.is_none());
    }

    #[test]
    fn test_extract_unchecked_tasks_no_goals_section() {
        let content = r#"# Entry

## Work Accomplished
- Did stuff
"#;

        let unchecked = extract_unchecked_tasks(content);
        assert!(unchecked.is_none());
    }
}
