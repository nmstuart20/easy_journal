# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based command-line journal application (`easy_journal`) that integrates with mdbook for viewing entries. It features:
- Daily journal entry creation with templating
- Integration with Apple Reminders (macOS) and Google Tasks
- Smart task carryover from previous entries
- Web server for mobile access
- Automatic mdbook SUMMARY.md generation with reverse chronological ordering

## Development Commands

### Build and Run
```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Install locally
cargo install --path .

# Run the journal CLI
cargo run -- [COMMAND]
```

### Testing and Quality
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Lint with clippy (fails on warnings in CI)
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Journal Commands (after installation)
```bash
# Create today's entry
journal
# or
journal new

# Create entry for specific date
journal new --date 2025-12-29

# Initialize journal structure
journal init

# Start web server for mobile access (port 3030)
journal serve

# Authenticate with Google Tasks
journal auth google
```

## Architecture

### Core Module Structure

**Entry Flow**: `main.rs` → `commands/` → `journal/` modules
- CLI defined in `main.rs` with clap derive macros
- Commands are separate modules in `commands/`
- Journal logic is in `journal/` modules

### Key Architectural Patterns

**Summary Management** (`journal/summary.rs`):
- Parses SUMMARY.md into a node-based AST structure
- Maintains user content above `---` separator, auto-generated content below
- Inserts entries in reverse chronological order (newest first)
- Structure: Year headers → Month entries → Day entries (nested 2-level list)

**Template System** (`journal/template.rs`):
- Variables: `{{date}}`, `{{day_of_week}}`, `{{year}}`, `{{month}}`, `{{month_num}}`, `{{day}}`, `{{reminders}}`
- Smart content injection: carries over unchecked tasks from "Goals for Today" in previous entry
- Carries over "Tomorrow's Focus" section from previous entry
- Converts regular list items to checkboxes automatically

**Task Carryover Logic** (`journal/entry.rs` + `journal/parser.rs`):
- Searches back up to 30 days for the most recent entry
- Extracts unchecked tasks from "Goals for Today" section using `parser::extract_unchecked_tasks()`
- Extracts "Tomorrow's Focus" section using `parser::extract_section()`
- Injects combined content into new entry's "Goals for Today" section

**Reminders Integration** (`journal/reminders.rs` + `journal/google_tasks.rs`):
- Fetches Apple Reminders and Google Tasks **concurrently** using `tokio::join!`
- Apple Reminders: Uses optimized AppleScript that fetches all reminders in a single IPC call
- Google Tasks: Uses OAuth2 with stored tokens at `~/.easy_journal_tokens.json`
- Both sources are non-blocking: failures print warnings but don't stop entry creation
- Results merged with section headers (`### Apple Reminders`, `### Google Tasks`)

**Web Server** (`commands/serve.rs`):
- Axum-based REST API on port 3030
- GET `/api/entry?date=YYYY-MM-DD` - Fetch or preview entry (includes reminder fetching)
- POST `/api/entry` - Save entry with auto-SUMMARY.md update
- Inline HTML with mobile-responsive UI and loading modal for reminder fetching

### Google OAuth Setup

Google Tasks requires OAuth credentials stored in environment variables:
- `GOOGLE_CLIENT_ID` - OAuth client ID
- `GOOGLE_CLIENT_SECRET` - OAuth client secret
- Tokens stored in `~/.easy_journal_tokens.json` after running `journal auth google`

### macOS-Specific Features

**Apple Reminders**: Requires automation permissions granted in System Settings → Privacy & Security → Automation. The AppleScript in `reminders.rs` is optimized to fetch all incomplete reminders in a single IPC call with a 120-second timeout.

## File Organization

```
journal/
├── YYYY/               # Year directories
│   ├── MM/            # Month directories
│   │   ├── README.md  # Month overview
│   │   └── DD.md      # Daily entries
├── SUMMARY.md         # mdbook navigation (reverse chronological)
└── README.md          # Journal homepage

template.md            # Entry template with variables
book.toml             # mdbook configuration (points to journal/ as src)
```

## Important Implementation Notes

- **Async Context**: Entry creation uses `async` because it fetches reminders concurrently. Use `tokio::spawn_blocking` for blocking Apple Reminders calls.
- **Error Handling**: Reminder/task fetching failures should be non-fatal (print warnings, return `None`). Only fail hard on filesystem/parser errors.
- **Date Parsing**: All dates use `NaiveDate` from chrono in `YYYY-MM-DD` format.
- **SUMMARY.md**: Preserve all user content above the `---` separator. Generated content is always reverse chronological (newest first).
- **Edition**: Uses Rust 2024 edition (see `Cargo.toml`).

## Testing Strategy

Tests are embedded in module files using `#[cfg(test)]`:
- Template variable substitution tests
- Task extraction and checkbox conversion tests
- Summary parsing and insertion order tests
- Reminder formatting tests
- Platform-specific tests with `#[cfg(target_os = "macos")]`
