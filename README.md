# Browser Scheduler

A cross-platform desktop application for scheduling browsers to open and close at specific times. Perfect for automating your daily browsing routines, managing work sessions, or scheduling time-limited access to websites.

![Browser Scheduler](icon-design.svg)

## Features

### 🌐 Multi-Browser Support
- **Automatic Detection**: Detects installed browsers (Chrome, Firefox, Edge, Safari, Brave, Opera, Chromium, LibreWolf)
- **Default Browser**: Automatically selects your system's default browser
- **Profiles (basic)**: Optional free-text profile name/directory for launch; full profile management is [planned](BACKLOG.md)

### ⏰ Flexible Scheduling
- **Natural Language Input**: Enter schedules like "January 31st from 9am to 11am ET" or "tomorrow at 2pm JST"
- **Schedule timezone**: Pick an IANA zone (or let NL abbreviations like ET/PT/CET set it); wall times are entered in that zone
- **UTC storage**: Instants are stored as UTC; repeats keep the same local wall clock across DST in the schedule zone
- **Local display**: Task cards show your local time plus the schedule-zone time when they differ
- **Repeating Tasks**: Daily, weekly, or monthly recurring schedules
- **Close behavior**: Reliable tab close on macOS; Windows/Linux are best-effort (window title / optional close-all). True cross-platform tab close is [on hold](BACKLOG.md)

### 🔒 Security Features
- **Input Validation**: Server-side validation prevents malicious URLs and path traversal
- **Content Security Policy**: Enabled CSP to prevent XSS attacks
- **Sanitized Execution**: Protected against AppleScript injection and command injection
- **URL Scheme Filtering**: Blocks dangerous URL schemes (javascript:, data:, file:, etc.)

### 🎯 Smart Task Management
- **Real-time Updates**: Event-based UI updates for instant status changes
- **Automatic Scheduling**: Background scheduler runs automatically on startup
- **Task Status**: Monitor tasks (Active, Completed, Failed, Disabled)
- **Failure logging**: Last error on the task card plus a recent execution log
- **Pause / Retry**: Pause active tasks; resume disabled or retry failed without deleting

### 🎨 Modern UI
- **Responsive Design**: Clean, intuitive interface built with React and Tailwind CSS
- **Quick Time Entry**: Natural language date parsing for faster task creation
- **Visual Feedback**: Color-coded status indicators and tooltips
- **Dark mode**: [Planned](BACKLOG.md) — styles exist; theme switching not wired yet

See [BACKLOG.md](BACKLOG.md) for deferred work (profile management, dark mode, major close improvements).

## Installation

**Deno is preferred** for the frontend toolchain: it still fetches packages from
the npm registry, but **does not run install/lifecycle scripts by default**,
which avoids the common supply-chain attack vector of auto-executed
`postinstall` hooks. Node/npm remains supported for cross-compatibility.

### Prerequisites

- **Deno** 2.x (Arch: `pacman -S deno`) *or* **Node.js** 18+
- **Rust** stable via rustup (Arch: `pacman -S rustup` then `rustup default stable`)
- **Tauri Linux deps** (Arch):

```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg
```

Optional but recommended: install the Tauri CLI with Cargo so you never need the
npm-based CLI binary:

```bash
cargo install tauri-cli --version "^2.0.0" --locked
```

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/vcostin/do-in-time.git
cd do-in-time
```

2. Install frontend dependencies (pick one):
```bash
deno install          # preferred
# or: npm install
```

3. Run in development mode (pick one):
```bash
# Preferred if cargo-tauri is installed
cargo tauri dev

# Or via package-manager task runners
deno task tauri dev
# or: npm run tauri dev
```

`tauri.conf.json` starts Vite through `scripts/frontend.sh`, which prefers Deno
and falls back to npm automatically.

4. Build for production:
```bash
cargo tauri build
# or: deno task tauri build
# or: npm run tauri build
```

The built application will be in `src-tauri/target/release/`.

### Frontend-only (Vite)

Useful for UI work without launching the native shell:

```bash
deno task dev
# or: npm run dev
# or: sh scripts/frontend.sh dev
```

## Usage

### Creating a Scheduled Task

1. **Click "+ New Task"** to open the task creation form

2. **Fill in task details:**
   - **Task Name**: Descriptive name (e.g., "Morning News Check")
   - **Browser**: Select from detected browsers
   - **URL**: Website to open (optional)
   - **Browser Profile**: Specific profile to use (optional)

3. **Set schedule using one of two methods:**

   Choose a **Schedule timezone** (defaults to your system zone). Start/close times are wall clocks in that zone and are saved as UTC.

   **Option A - Natural Language (Quick):**
   - Enter: "tomorrow from 9am to 5pm" (uses the selected schedule timezone)
   - Enter: "next Friday at 2pm PT"
   - Enter: "January 31st from 9am to 11am ET"
   - Enter: "March 15 2026 9:00 JST"
   - Zone abbreviations (ET, PT, CET, JST, …) update the schedule timezone; failed parses show an error

   **Option B - Manual (Precise):**
   - Select **Start Time** from datetime picker (in the schedule timezone)
   - Select **Close Time** (optional)

4. **Configure repeating (optional):**
   - Enable "Repeat task"
   - Choose interval: Daily, Weekly, Monthly
   - Set end conditions: after N occurrences or by date

5. **Click "Create Task"** to save

### Closing browsers

- **macOS**: closes tabs whose URL contains the scheduled URL (AppleScript).
- **Windows / Linux**: closes windows whose title contains the URL host (best-effort). If nothing matches, enable **Allow close all browser instances** to terminate that browser’s processes as a fallback. Without a URL, close always requires that option.

### Managing Tasks

- **Edit**: Click the "Edit" button on any task
- **Delete**: Click "Delete" to remove a task
- **Pause / Resume / Retry**: Pause active tasks; resume disabled ones or retry after a failure
- **Log**: Expand recent open/close attempts and error messages
- **Status**: Tasks show real-time status (Active, Completed, Failed, Disabled)

### Scheduler Control

The scheduler starts automatically on application launch. You can:
- **Stop Scheduler**: Click the scheduler status indicator
- **Start Scheduler**: Click again to restart
- **Monitor Status**: Green = Running, Red = Stopped

## Architecture

### Backend (Rust)
- **Tauri Framework**: Native desktop application framework
- **SQLx**: Type-safe SQL database access with SQLite
- **Tokio**: Async runtime for task scheduling
- **Chrono**: Date/time handling with timezone support

### Frontend (TypeScript/React)
- **React 19**: Modern UI library
- **TypeScript**: Type-safe development (Deno toolchain)
- **Tailwind CSS**: Utility-first styling
- **Vite**: Fast build tool and dev server
- **Chrono-node**: Natural language date parsing

### Security
- **Input Validation**: All user inputs validated server-side
- **Parameterized Queries**: SQL injection prevention
- **Command Sanitization**: Protected against command injection
- **CSP Enabled**: Cross-site scripting prevention

## Database Schema

Tasks and settings are stored locally in SQLite. Current shape (see `src-tauri/src/db/schema.rs`):

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    browser TEXT NOT NULL,
    browser_profile TEXT,
    url TEXT,
    allow_close_all INTEGER NOT NULL DEFAULT 0,
    start_time TEXT NOT NULL,
    close_time TEXT,
    timezone TEXT NOT NULL,
    repeat_interval TEXT,
    repeat_end_after INTEGER,
    repeat_end_date TEXT,
    execution_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK(status IN ('active', 'completed', 'failed', 'disabled')),
    next_open_execution TEXT,
    next_close_execution TEXT,
    last_error TEXT,
    last_execution_at TEXT
);

CREATE TABLE task_execution_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    success INTEGER NOT NULL,
    message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY
);
```

Schema upgrades run on startup via `schema_migrations` (for example adding `last_error` / `disabled` on older databases).

## Database Location

- **Windows**: `%APPDATA%\do-in-time\data.db`
- **macOS**: `~/Library/Application Support/do-in-time/data.db`
- **Linux**: `~/.local/share/do-in-time/data.db`

## Development

### Project Structure

```
do-in-time/
├── src/                      # Frontend React code
│   ├── components/          # React components
│   ├── hooks/               # Custom React hooks
│   ├── services/            # API services
│   ├── types/               # TypeScript types
│   └── utils/               # Utility functions
├── src-tauri/               # Backend Rust code
│   ├── src/
│   │   ├── commands/       # Tauri commands (API endpoints)
│   │   ├── core/           # Business logic
│   │   ├── db/             # Database layer
│   │   └── utils/          # Utility modules
│   └── icons/              # Application icons
├── BACKLOG.md               # Deferred / on-hold work
└── README.md
```

### Running Tests

```bash
# Rust backend tests
cd src-tauri
cargo test

# Frontend timezone / datetime util tests
deno task test:utils
```

### Code Style

- **Rust**: Uses `rustfmt` and `clippy`
- **TypeScript**: `deno task check` / `npm run check` (full `src/` via `tsc`)

## Troubleshooting

### Browser Not Detected
- Ensure the browser is installed in the default location
- Check if the browser executable is in your PATH

### Task Not Executing
- Verify the scheduler is running (green indicator)
- Check task status — it should be **Active** (not Disabled, Failed, or Completed)
- Ensure the next open/close time is in the future (or use **Retry** after a failure)
- On the task card, check **Last error** and expand **Log** for recent open/close attempts
- If notifications are enabled in Settings, failure alerts include the error message

### Database Issues
If you encounter database errors:
1. Close the application
2. Delete the database file (see Database Location above)
3. Restart the application (fresh database will be created)

## Credits

Built with assistance from Claude (Anthropic's AI assistant) for code implementation and security hardening.
