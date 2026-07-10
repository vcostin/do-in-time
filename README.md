# Browser Scheduler

A cross-platform desktop application for scheduling browsers to open and close at specific times. Perfect for automating your daily browsing routines, managing work sessions, or scheduling time-limited access to websites.

![Browser Scheduler](icon-design.svg)

## Features

### Multi-browser support
- **Automatic detection**: Chrome, Firefox, Edge, Safari, Brave, Opera, Chromium, LibreWolf
- **Default browser**: Prefills your system default when available

### Flexible scheduling
- **Natural language**: e.g. "January 31st from 9am to 11am ET" or "tomorrow at 2pm JST"
- **Schedule timezone**: IANA zone picker; NL abbreviations (ET, PT, CET, JST, …) update it
- **UTC storage**: Instants stored as UTC; repeats keep local wall clock across DST in the schedule zone
- **Local display**: Task cards show operator-local time plus schedule-zone time when they differ
- **12 / 24-hour clock**: Settings toggle (default 24h) for cards, Activity, and date pickers
- **Repeating tasks**: Daily, weekly, or monthly with end-after / end-by-date
- **Close behavior**: Reliable tab close on macOS; Windows/Linux are best-effort (window title, optional close-all). Soft close misses log and clear the close slot without failing the task. True cross-platform tab close is [on hold](BACKLOG.md)

### Task management
- **List and calendar**: List view or Day / Week / Month calendar of upcoming open/close events
- **Pages (not modals)**: Hash routes for home, calendar, new/edit task, Activity, Settings
- **Unsaved leave guard**: Warns before leaving a task form with unsaved edits
- **Statuses**: Active, Completed, Failed, Disabled
- **Failure logging**: Last error on the card plus a per-task execution log
- **Activity feed**: Cross-task open/close history (optional failures-only filter)
- **Pause / Retry**: Pause active tasks; resume disabled or retry failed without deleting

### Settings
- Minimize to tray / start minimized
- Desktop notifications for open/close (and failures / close misses)
- Launch at login (auto-start)
- 12 / 24-hour time format

### Security
- **Command validation**: Task create/update inputs are validated in the Rust backend (Tauri IPC), not only in the UI
- **Content Security Policy**: CSP on the webview to limit XSS impact
- **Sanitized execution**: Guards against AppleScript / shell injection in launch and close paths
- **URL scheme filtering**: Blocks dangerous schemes (`javascript:`, `data:`, `file:`, etc.)

### UI
- React + Tailwind; sticky header while scrolling
- react-datepicker for start/close times (respects 12/24h setting)
- Compact task cards; color-coded status
- **Dark mode**: [Planned](BACKLOG.md) — `dark:` styles exist; theme switching not wired yet

See [BACKLOG.md](BACKLOG.md) for deferred work (AppImage on Arch, profile management, dark mode, major close improvements).

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

For **scheduled close on Linux** (title match), also install one of:

```bash
sudo pacman -S --needed wmctrl   # preferred
# or: sudo pacman -S --needed xdotool
```

Without `wmctrl`/`xdotool`, Linux close falls back to “close missed” (or process kill if **Allow close all** is enabled). On **Wayland**, those tools are X11-oriented and often find no windows — expect soft close misses unless you use allow-close-all or run under X11.

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

Linux packages default to `.deb` and `.rpm`. AppImage is omitted because Arch’s
gdk-pixbuf 2.44+ (glycin) no longer provides the classic loaders tree that
Tauri’s linuxdeploy GTK plugin expects; that broke after it used to work on
older gdk-pixbuf. To force an AppImage anyway: `deno task tauri build -- --bundles appimage`.

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
- **Windows**: closes windows whose title contains the URL host (best-effort). If nothing matches, enable **Allow close all browser instances** to terminate that browser’s processes. Without a URL, close always requires that option.
- **Linux**: same title-match idea via **`wmctrl`** (preferred) or **`xdotool`**. If neither tool is installed, or no window title matches (common on **Wayland**), the run is a **soft close miss**: logged as “Close missed”, close slot cleared, task not marked Failed. **Allow close all** uses `pkill -f` on a short browser name as a last resort (can match unrelated processes — see [BACKLOG.md](BACKLOG.md)).

### Managing Tasks

- **Edit**: Click the "Edit" button on any task
- **Delete**: Click "Delete" to remove a task
- **Pause / Resume / Retry**: Pause active tasks; resume disabled ones or retry after a failure
- **Log**: Expand recent open/close attempts and error messages on a task
- **Activity**: Header button opens a cross-task activity feed (optional failures-only filter)
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
- **IPC validation**: Create/update payloads are checked in Rust command handlers before DB writes
- **Parameterized queries**: SQL injection prevention
- **Command sanitization**: Launch/close paths avoid shell injection
- **CSP**: Webview content security policy
- **Plugin permissions**: Notifications and auto-start are allowed in `src-tauri/capabilities/default.json`; OS may still deny them (settings stay saved; auto-start surfaces a warning if apply fails)

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

Release builds and debug/`tauri dev` builds use **separate** SQLite files in the same app data directory:

| Build | Filename |
|-------|----------|
| Release | `data.db` |
| Debug / `tauri dev` | `dev-data.db` |

Directories:

- **Windows**: `%APPDATA%\do-in-time\`
- **macOS**: `~/Library/Application Support/do-in-time/`
- **Linux**: `~/.local/share/do-in-time/`

Example (Linux): `~/.local/share/do-in-time/data.db` (release) vs `~/.local/share/do-in-time/dev-data.db` (dev).

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

### Browser not detected
- Ensure the browser is installed in the default location
- Check if the browser executable is in your PATH

### Task not executing
- Verify the scheduler is running (green indicator)
- Check task status — it should be **Active** (not Disabled, Failed, or Completed)
- Ensure the next open/close time is in the future (or use **Retry** after a failure)
- On the task card, check **Last error** and expand **Log** for recent open/close attempts
- If notifications are enabled in Settings, failure alerts include the error message

### Close missed (Linux / Windows)
- Soft miss means no matching window/tab was found; the task is not Failed
- Linux: install `wmctrl` or `xdotool`; on Wayland, title close often fails — try X11 (`GDK_BACKEND=x11`) or enable **Allow close all** knowing it kills by process name
- Confirm the browser window title contains the URL host (or path fragment used as the needle)
- Check **Log** / Activity for the exact “Close missed” message

### Notifications or auto-start not applying
- Toggle the setting in **Settings**; auto-start failures keep the saved preference and show a warning
- Confirm the OS allowed notifications / login items for the app
- Capabilities include `notification:default` and `autostart:default`; missing OS permission is still a soft failure by design

### Database issues
Prefer diagnosing with **Last error**, **Log**, and **Activity** before wiping data.

If the app will not start or the schema is clearly corrupt:
1. Close the application
2. Back up then delete the DB file for **this** build (release `data.db` vs debug `dev-data.db` — see Database Location)
3. Restart (a fresh database is created)

In-app “nuke all data” is [not shipped](BACKLOG.md).

### Tray restore: title-bar buttons unclickable (Linux / KDE)
After hiding to the tray and showing again, minimize / maximize / close can stop responding until you double-click the title bar. This is a known Tauri + Wayland decoration bug; the app toggles window `resizable` on show/focus as a workaround. If it still happens, try launching under X11 (`GDK_BACKEND=x11`) or update Tauri when upstream fixes land.

## Credits

Built with assistance from Claude (Anthropic's AI assistant) for code implementation and security hardening.
