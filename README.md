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
- `.gitignore` - prevents committing tokens and build artifacts
- `.env.example` - template for your API tokens and credentials

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

#### Include GitHub Issues and PRs
```bash
easy_journal new --github
```

This will automatically fetch and include:
- Assigned issues
- Created issues
- Assigned pull requests
- Pull requests where you're requested as reviewer

#### Include GitLab Issues and MRs
```bash
easy_journal new --gitlab
```

This will automatically fetch and include:
- Assigned issues
- Created issues
- Assigned merge requests
- Merge requests where you're requested as reviewer

#### Use Both GitHub and GitLab
```bash
easy_journal new --github --gitlab
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

## GitHub and GitLab Integration

Easy Journal can automatically pull your assigned issues, merge requests, and review requests from GitHub and GitLab and add them to your daily entries.

### Quick Setup with .env File (Recommended)

The easiest way to manage your API tokens is using a `.env` file. If you used `easy_journal init`, the `.env.example` file is already created for you!

1. Copy the example file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and add your tokens:
   ```bash
   # GitHub
   GITHUB_TOKEN=ghp_your_token_here

   # GitLab
   GITLAB_TOKEN=glpat-your_token_here

   # Google Tasks (optional)
   GOOGLE_CLIENT_ID=your_client_id
   GOOGLE_CLIENT_SECRET=your_client_secret
   ```

3. The `.env` file is already in `.gitignore` and won't be committed

4. Use the flags when creating entries:
   ```bash
   easy_journal new --github --gitlab
   ```

### GitHub Setup

1. Create a personal access token:
   - Go to GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Generate new token with these scopes:
     - `repo` (for private repos)
     - `read:org` (for organization repos)
   - Or use fine-grained tokens with:
     - Repository access: Read access to issues and pull requests

2. Add to `.env` file (recommended):
   ```bash
   GITHUB_TOKEN=ghp_your_token_here
   ```

   Or set as environment variable:
   ```bash
   export GITHUB_TOKEN="your_token_here"
   ```

3. Use the --github flag when creating entries:
   ```bash
   easy_journal new --github
   ```

### GitLab Setup

1. Create a personal access token:
   - Go to GitLab → Preferences → Access Tokens
   - Create token with `read_api` scope

2. Add to `.env` file (recommended):
   ```bash
   GITLAB_TOKEN=glpat-your_token_here
   # Optional: for self-hosted GitLab instances
   GITLAB_HOST=https://gitlab.example.com
   ```

   Or set as environment variables:
   ```bash
   export GITLAB_TOKEN="your_token_here"
   export GITLAB_HOST="https://gitlab.example.com"
   ```

3. Use the --gitlab flag when creating entries:
   ```bash
   easy_journal new --gitlab
   ```

### What Gets Included

When you use --github or --gitlab, your daily entry will automatically include:

- **Assigned Issues**: Issues assigned to you
- **Created Issues**: Issues you created (even if not assigned to you)
- **Assigned MRs/PRs**: Merge requests or pull requests assigned to you
- **Review Requests**: MRs/PRs where you're requested as a reviewer

Each item includes:
- Title with link
- Repository/project name
- Issue/MR number
- Labels (if any)
- Due date (if set)

Items are formatted as markdown checkboxes so you can track them in your journal.

### Example Output

```markdown
## Reminders
{{reminders}}

### GitHub

#### Review Requests
- [ ] [owner/repo] Add new feature (#456) [feature] [priority-high]
      https://github.com/owner/repo/pull/456

#### Assigned Issues
- [ ] [owner/repo] Fix login bug (#123) [bug] [urgent] - Due: 2026-01-15
      https://github.com/owner/repo/issues/123

### GitLab

#### Assigned MRs
- [ ] [group/project] Update documentation (!789) [docs]
      https://gitlab.com/group/project/-/merge_requests/789
```

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