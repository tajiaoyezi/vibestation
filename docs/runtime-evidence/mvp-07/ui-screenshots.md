# MVP-07 UI Screenshots - Description Log

Since this is a CLI/automated session, runtime screenshots would require a running
Tauri dev app with a display server. Below are the expected UI states documented:

## Screenshot 1: Empty State (No Git Repo)
When a workspace without a `.git` folder is opened, the GitLog panel shows:
- "No git repository found" placeholder text
- Secondary subtitle: "Open a directory containing a .git folder"

## Screenshot 2: Git Log List with Commits
When a workspace with git is opened, the panel shows:
- Search/filter bar with two inputs (message, author)
- Paginated commit list with short_sha, time, message, author, branch/tag labels
- "Load more" button at the bottom when hasMore=true
- Clicking a commit shows detail view below the list

## Screenshot 3: Commit Detail View
When a commit is selected:
- Detail section slides up showing full SHA, author, committer, date, message
- File change list with status indicators (A=green, M=yellow, D=red, R=blue)
- Parent commit SHAs linked

## CSS Classes Applied (Calm Studio design tokens)
All styles use the project's `:root` CSS custom properties:
- `--bg-0`, `--bg-1`, `--bg-2` for backgrounds
- `--text-1` through `--text-4` for text hierarchy
- `--accent`, `--accent-soft` for interactive highlights
- `--font-mono`, `--font-ui` for typography
- `--space-*` for spacing rhythm
- `--r-*` for border radii
- `--dur-*`, `--ease` for transitions