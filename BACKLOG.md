# Backlog

Tracked future work. Items here are **not** implemented (or only partially) and should not be treated as shipped product promises.

Status legend: `planned` · `on-hold` · `in-progress` · `done`

---

## On hold

### [major] Reliable schedule close (Windows / Linux tab close)

**Status:** `on-hold`  
**Why on hold:** Closing a specific browser **tab** by URL is not reliably available on Windows or Linux without browser-specific automation (Chrome DevTools Protocol, extensions, or similar). macOS can close matching tabs via AppleScript; Win/Linux today only offer best-effort **window title** matching and an optional **close-all processes** fallback.

**Current behavior (shipped, limited):**
- macOS: close tabs whose URL contains the task URL
- Windows / Linux: close windows whose title contains the URL host; process kill only if `allow_close_all`

**Future directions (when unblocked):**
- Optional CDP / remote-debugging launch mode for Chromium-family browsers
- Browser extension + native messaging for tab close by URL
- Clearer in-app messaging when close cannot target a single tab (esp. Wayland)

**Acceptance (when resumed):**
- Closing a scheduled session does not require killing the entire browser by default
- Documented, tested path per supported OS/browser family

---

## Planned

### Failure logging / last error

**Status:** `planned`  
**Why:** When a task becomes `Failed`, the UI only shows the status — not *why* (launch failed, close failed, validation, etc.). Needed to diagnose and fix issues in later iterations.

**Today:** Errors are returned from `TaskExecutor` / launcher and the task is marked `Failed`; no `last_error` (or history) is persisted or shown.

**Future (minimal first, then optional history):**
- Persist `last_error` (and `last_execution_at`) on the task when open/close fails
- Clear `last_error` on successful run or when the user retries / reactivates
- Show the message on the task card (and optionally in a notification body)
- Later: append-only execution log table if a full history UI is needed

**Acceptance:**
- A failed open or close leaves a human-readable reason visible in the app
- Success clears or supersedes the previous error
- No secrets (full profile paths with credentials, etc.) in stored messages

### Profile management

**Status:** `planned`  
**Today:** Free-text profile field passed as launch args (Chrome/Edge profile directory, Firefox profile name). No discovery or validation against installed profiles.

**Future:**
- Enumerate installed Chrome/Chromium/Edge/Firefox/LibreWolf profiles
- Picker in the task form (with free-text fallback)
- Surface invalid/missing profile before schedule fire when possible

**Acceptance:**
- User can select a detected profile without typing internal names
- Launch still works when the user opts for a custom profile string

### Dark mode

**Status:** `planned`  
**Today:** Tailwind `dark:` styles exist, but nothing applies a `dark` class or follows `prefers-color-scheme`, so dark styles never activate.

**Future:**
- System preference and/or explicit light/dark/system toggle
- Persist choice in settings
- Ensure tray/settings/task UI remain readable in both themes

**Acceptance:**
- Dark theme is reachable without manual DOM hacks
- Preference survives app restart

### Pause / Disabled + retry Failed

**Status:** `planned`  
**Today:** Only `active` / `completed` / `failed`. Failed tasks stay failed until schedule times are edited.

**Future:**
- `disabled` (or paused) status so routines can be stopped without deleting
- Retry / re-enable control for Failed tasks without changing times

---

## Notes

- Prefer updating this file when scope or status changes, rather than re-advertising unfinished items as Features in the README.
- Related analysis: intention-vs-implementation review (local Cursor canvas).
- **Next iteration priority (suggested):** failure logging → pause/retry → (then profile / dark mode). Close remains on-hold.