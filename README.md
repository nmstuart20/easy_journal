# Easy Journal

A an over-engineered command-line journal system that uses Rust mdbook. Create, organize, and browse your daily journal entries.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70 or later)
- [mdbook](https://rust-lang.github.io/mdBook/guide/installation.html) (for viewing your journal)

### Installation

**Option 1: Install from source (recommended)**

1. Install the journal CLI:
   ```bash
   cargo install --path .
   ```
2. Install mdbook
   ```bash
   cargo install mdbook
   ```
3. Create a directory for your journal and initialize:
   ```bash
   mkdir my-journal
   cd my-journal
   easy_journal init
   ```

The `init` command automatically creates everything you need:
- `book.toml` - mdbook configuration
- `journal/` directory with README.md and SUMMARY.md
- `template.md` - customizable daily entry template

### Usage

#### Create Today's Entry
```bash
easy_journal
```
or
```bash
easy_journal new
```

#### Create Entry for Specific Date
```bash
easy_journal new --date 2025-12-29
```

#### View Your Journal (with mdbook)
```bash
mdbook serve --open
```

This will build and serve your journal at `http://localhost:3000` with live-reload.

#### View And Edit Your Journal (with axum)
```bash
easy_journal serve
```

This will build and serve your journal at `http://0.0.0.0:3030` with live-reload.

## Template Customization

Your daily entries are created from `template.md`. Edit this file to customize your journal structure:

### Available Template Variables

- `{{date}}` - Full date (YYYY-MM-DD)
- `{{day_of_week}}` - Day name (Monday, Tuesday, etc.)
- `{{year}}` - Four-digit year
- `{{month}}` - Full month name
- `{{month_num}}` - Month number (01-12)
- `{{day}}` - Day number (01-31)

### Example Template

```markdown
# {{date}} - {{day_of_week}}

## Goals for Today
- [ ]
- [ ]

## Work Accomplished
-

## Reflections
-

---
**Mood**:
**Energy**:
```

## Project Structure

```
rusty_journal/
├── journal/              # Your journal entries
│   ├── SUMMARY.md       # Auto-generated navigation
│   ├── README.md        # Journal home page
│   └── 2025/           # Year folders
│       └── 12/         # Month folders
│           └── 29.md   # Daily entries
├── template.md          # Entry template
├── book.toml           # mdbook configuration
└── src/                # Rust source code
```

## How It Works

1. **Entry Creation**: When you run `easy_journal`, it:
   - Creates year/month directories if needed
   - Generates month README files
   - Applies your template with date variables
   - Updates SUMMARY.md with proper hierarchy
   - Opens the entry in your editor

2. **Organization**: Entries are organized as `journal/YYYY/MM/DD.md` and automatically added to the navigation in reverse chronological order (newest first).

3. **Rendering**: mdbook transforms your markdown files into a searchable static website.

## Tips

- Set your preferred editor: `export EDITOR=nano` or `export VISUAL=code`
- Customize `template.md` to match your journaling style

## Troubleshooting

**Editor doesn't open?**
- Set the `EDITOR` or `VISUAL` environment variable
- The tool will try vscode, vim, nano, or vi as fallbacks

**Entries not showing in mdbook?**
- Make sure you've run `easy_journal init` first
- Check that `book.toml` points to `src = "journal"`

## License

MIT License - feel free to use this project for your personal journaling needs.