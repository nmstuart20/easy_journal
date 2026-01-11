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

Use the `easy_journal` command to create daily entries:

```bash
# Create/open today's entry
easy_journal

# Create entry for specific date
easy_journal new --date 2025-12-29

# Initialize the journal structure
easy_journal init
```

## Navigation

Browse your entries by year and month using the sidebar navigation.

## Customization

Edit the `template.md` file in the project root to customize your daily entry template.

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

**Sleep Score(1-100)**:

**Sleep Time**:

**Mood(1-10)**:

**Energy Level(1-10)**:

**Hours Worked**:
"#;
        fs::write(&config.template_path, template_content)?;
        println!("✓ Created template.md");
    }

    // Create month_template.md if it doesn't exist
    if !config.month_template_path.exists() {
        let month_template_content = r#"# {{month}} {{year}}

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
        fs::write(&config.month_template_path, month_template_content)?;
        println!("✓ Created month_template.md");
    }

    // Create year_template.md if it doesn't exist
    if !config.year_template_path.exists() {
        let year_template_content = r#"# Year in Review: {{year}}

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
        fs::write(&config.year_template_path, year_template_content)?;
        println!("✓ Created year_template.md");
    }

    // Create .gitignore if it doesn't exist
    let gitignore_path = Path::new(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = r#"/target
.easy_journal_tokens.json
.env
credentials.json
book/
"#;
        fs::write(gitignore_path, gitignore_content)?;
        println!("✓ Created .gitignore");
    }

    // Create .env.example if it doesn't exist
    let env_example_path = Path::new(".env.example");
    if !env_example_path.exists() {
        let env_example_content = r#"# Easy Journal Environment Variables
# Copy this file to .env and fill in your credentials
# The .env file is already in .gitignore and will not be committed

# Google Tasks OAuth Credentials
# Get these from: https://console.cloud.google.com/
GOOGLE_CLIENT_ID=your_google_client_id_here
GOOGLE_CLIENT_SECRET=your_google_client_secret_here

# GitHub Personal Access Token
# Create at: https://github.com/settings/tokens
# Required scopes: repo, read:org (or use fine-grained tokens with read access to issues/PRs)
GITHUB_TOKEN=ghp_your_github_token_here

# GitLab Personal Access Token
# Create at: https://gitlab.com/-/user_settings/personal_access_tokens
# Required scope: read_api
GITLAB_TOKEN=glpat-your_gitlab_token_here

# GitLab Host (optional - defaults to https://gitlab.com)
# For self-hosted GitLab instances:
# GITLAB_HOST=https://gitlab.example.com
"#;
        fs::write(env_example_path, env_example_content)?;
        println!("✓ Created .env.example");
    }

    println!("\n🎉 Journal repository initialized successfully!");
    println!("\nNext steps:");
    println!("  1. Copy .env.example to .env and add your API tokens (optional)");
    println!("  2. Run 'easy_journal' to create your first entry");
    println!("  3. Run 'mdbook serve' to view your journal in a browser");
    println!("  4. Customize 'template.md' to personalize your daily entries");
    println!("  5. Customize 'month_template.md' for monthly reviews");
    println!("  6. Customize 'year_template.md' for yearly reviews");

    Ok(())
}
