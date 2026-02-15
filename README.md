# Browser Scheduler

A cross-platform desktop application for scheduling browsers to open and close at specific times. Perfect for automating your daily browsing routines, managing work sessions, or scheduling time-limited access to websites.

![Browser Scheduler](icon-design.svg)

## Features

### 🌐 Multi-Browser Support
- **Automatic Detection**: Detects installed browsers (Chrome, Firefox, Edge, Safari, Brave, Opera)
- **Profile Management**: Support for browser profiles (Chrome profiles, Firefox profiles)
- **Default Browser**: Automatically selects your system's default browser

### ⏰ Flexible Scheduling
- **Natural Language Input**: Enter schedules like "January 31st from 9am to 11am ET"
- **Precise Timing**: Set exact start and close times for browser sessions
- **Timezone Support**: Full timezone handling with UTC storage and local display
- **Repeating Tasks**: Daily, weekly, or monthly recurring schedules

### 🔒 Security Features
- **Input Validation**: Server-side validation prevents malicious URLs and path traversal
- **Content Security Policy**: Enabled CSP to prevent XSS attacks
- **Sanitized Execution**: Protected against AppleScript injection and command injection
- **URL Scheme Filtering**: Blocks dangerous URL schemes (javascript:, data:, file:, etc.)

### 🎯 Smart Task Management
- **Real-time Updates**: Event-based UI updates for instant status changes
- **Task History**: Track execution history with success/failure logs
- **Automatic Scheduling**: Background scheduler runs automatically on startup
- **Task Status**: Monitor tasks (Active, Completed, Failed, Disabled)

###🎨 Modern UI
- **Dark Mode Support**: Automatic dark/light theme switching
- **Responsive Design**: Clean, intuitive interface built with React and Tailwind CSS
- **Quick Time Entry**: Natural language date parsing for faster task creation
- **Visual Feedback**: Color-coded status indicators and tooltips

## Installation

### Prerequisites

- **Node.js** (v18 or later)
- **Rust** (latest stable version)
- **npm** or **yarn**

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/do-in-time.git
cd do-in-time
```

2. Install dependencies:
```bash
npm install
```

3. Run in development mode:
```bash
npm run tauri dev
```

4. Build for production:
```bash
npm run tauri build
```

The built application will be in `src-tauri/target/release/`.

## Usage

### Creating a Scheduled Task

1. **Click "+ New Task"** to open the task creation form

2. **Fill in task details:**
   - **Task Name**: Descriptive name (e.g., "Morning News Check")
   - **Browser**: Select from detected browsers
   - **URL**: Website to open (optional)
   - **Browser Profile**: Specific profile to use (optional)

3. **Set schedule using one of two methods:**

   **Option A - Natural Language (Quick):**
   - Enter: "tomorrow from 9am to 5pm"
   - Enter: "next Friday at 2pm"
   - Enter: "January 31st from 9am to 11am ET"

   **Option B - Manual (Precise):**
   - Select **Start Time** from datetime picker
   - Select **Close Time** (optional)

4. **Configure repeating (optional):**
   - Enable "Repeat task"
   - Choose interval: Daily, Weekly, Monthly
   - Set end conditions: after N occurrences or by date

5. **Click "Create Task"** to save

### Managing Tasks

- **Edit**: Click the "Edit" button on any task
- **Delete**: Click "Delete" to remove a task
- **Status**: Tasks show real-time status (Active, Completed, Failed)
- **History**: View execution history for each task

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
- **React 18**: Modern UI library
- **TypeScript**: Type-safe development
- **Tailwind CSS**: Utility-first styling
- **Vite**: Fast build tool and dev server
- **Chrono-node**: Natural language date parsing

### Security
- **Input Validation**: All user inputs validated server-side
- **Parameterized Queries**: SQL injection prevention
- **Command Sanitization**: Protected against command injection
- **CSP Enabled**: Cross-site scripting prevention

## Database Schema

Tasks are stored locally in SQLite with the following structure:

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    browser TEXT NOT NULL,
    browser_profile TEXT,
    url TEXT,
    start_time TEXT NOT NULL,
    close_time TEXT,
    timezone TEXT NOT NULL,
    repeat_interval TEXT,
    repeat_end_after INTEGER,
    repeat_end_date TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_open_execution TEXT,
    last_close_execution TEXT,
    next_open_execution TEXT,
    next_close_execution TEXT
);
```

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
└── README.md
```

### Running Tests

```bash
# Rust backend tests
cd src-tauri
cargo test

# Frontend tests (if configured)
npm test
```

### Code Style

- **Rust**: Uses `rustfmt` and `clippy`
- **TypeScript**: ESLint and Prettier configured

## Troubleshooting

### Browser Not Detected
- Ensure the browser is installed in the default location
- Check if the browser executable is in your PATH

### Task Not Executing
- Verify the scheduler is running (green indicator)
- Check task status - it should be "Active"
- Ensure the scheduled time is in the future
- Check execution history for error messages

### Database Issues
If you encounter database errors:
1. Close the application
2. Delete the database file (see Database Location above)
3. Restart the application (fresh database will be created)

## Credits

Built with assistance from Claude (Anthropic's AI assistant) for code implementation and security hardening.
