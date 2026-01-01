use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::error::Result;

pub fn run(config: &Config) -> Result<()> {
    println!("Initializing journal repository...");

    // Create journal directory
    fs::create_dir_all(&config.journal_dir)?;
    println!("✓ Created journal directory: {:?}", config.journal_dir);

    // Create book.toml if it doesn't exist
    let book_toml_path = Path::new("book.toml");
    if !book_toml_path.exists() {
        let book_toml_content = r#"[book]
title = "Rusty Journal"
authors = ["Your Name"]
language = "en"
src = "journal"

[build]
build-dir = "book"
"#;
        fs::write(book_toml_path, book_toml_content)?;
        println!("✓ Created book.toml");
    }

    // Create README.md if it doesn't exist
    let readme_path = config.journal_dir.join("README.md");
    if !readme_path.exists() {
        let readme_content = r#"# Welcome to Your Journal

This is your personal daily journal, managed with Rust and built with mdbook.

## Getting Started

Use the `journal` command to create daily entries:

```bash
# Create/open today's entry
journal

# Create entry for specific date
journal new --date 2025-12-29

# Initialize the journal structure
journal init
```

## Navigation

Browse your entries by year and month using the sidebar navigation.

## Customization

Edit the `template.md` file in the project root to customize your daily entry template.

---

Happy journaling!
"#;
        fs::write(&readme_path, readme_content)?;
        println!("✓ Created README.md");
    }

    // Create SUMMARY.md if it doesn't exist
    let summary_path = config.journal_dir.join("SUMMARY.md");
    if !summary_path.exists() {
        let summary_content = r#"# Summary

[Introduction](README.md)

---
"#;
        fs::write(&summary_path, summary_content)?;
        println!("✓ Created SUMMARY.md");
    }

    // Create template.md if it doesn't exist
    if !config.template_path.exists() {
        let template_content = r#"# {{date}} - {{day_of_week}}

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

**Sleep Score(1-100)**:

**Sleep Time**:

**Mood(1-10)**:

**Energy Level(1-10)**:

**Hours Worked**:
"#;
        fs::write(&config.template_path, template_content)?;
        println!("✓ Created template.md");
    }

    println!("\n🎉 Journal repository initialized successfully!");
    println!("\nNext steps:");
    println!("  1. Run 'journal' to create your first entry");
    println!("  2. Run 'mdbook serve' to view your journal in a browser");
    println!("  3. Customize 'template.md' to personalize your daily entries");

    Ok(())
}
