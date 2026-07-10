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
- Persist launch session identity (PID / window / CDP target) so close can target what open started

**Acceptance (when resumed):**
- Closing a scheduled session does not require killing the entire browser by default
- Documented, tested path per supported OS/browser family

---

## Done

### Natural-language time / timezone honesty

**Status:** `done`  
- Schedule timezone picker (IANA); wall times entered in that zone
- NL abbreviations (ET, PT, JST, …) map to IANA zones — not ET-only hardcoding
- Instants stored as UTC; repeats use stored IANA zone for DST-safe math
- Failed NL parses surface an error; README matches behavior
- Task cards show operator-local time plus schedule-zone time when they differ

### Failure logging / last error + execution log

**Status:** `done`  
- `last_error` / `last_execution_at` on tasks
- Append-only `task_execution_log`
- Failed runs show the reason on the task card; **Log** expands recent entries
- Notifications include the error body when enabled

### Pause / Disabled + retry Failed

**Status:** `done`  
- `disabled` status; scheduler ignores disabled tasks
- **Pause** / **Resume** / **Retry** controls on the task card

### Softer close-failure handling

**Status:** `done`  
- Soft close misses (`CloseTargetNotFound`) log the miss and clear the close slot
- Repeating tasks stay Active; one-shots with nothing left become Completed
- Hard open/close failures still mark Failed

### Fix past one-shot “zombie” Active tasks

**Status:** `done`  
- `schedule_next_open_close` marks past one-shots with no next open/close as `Completed`
- `create_task` no longer forces Active over Completed

---

## Planned

### Harden Linux / Windows close scoping

**Status:** `planned`  
**Today:** Linux title-close ignores which browser was requested; `pkill -f` uses short names (`chrome`, `firefox`) that can match unrelated processes. Windows `taskkill` is process-image-wide when `allow_close_all` is set.

**Future:**
- Scope window matching to the task’s browser where possible
- Prefer exact process-name kills (`pkill -x` / equivalent) over substring cmdline matches
- Document required tools (`wmctrl` / `xdotool`) and Wayland limitations in-app and in the README

**Acceptance:**
- Close/kill paths do not intentionally target other browsers or unrelated processes
- Missing tools / unsupported environments surface a clear error, not a silent no-op

### Reschedule when repeat config changes

**Status:** `planned`  
**Today:** `update_task` only re-derives `next_*` when `start_time` / `close_time` change—not when `repeat_config` is added, removed, or edited.

**Future:**
- Re-run `schedule_next_open_close` when repeat interval / end conditions change
- Clear or recompute next close consistently with the new plan

**Acceptance:**
- Editing only the repeat settings updates upcoming executions without requiring a time edit

### Execution log retention

**Status:** `planned`  
**Today:** `task_execution_log` is append-only with no prune; UI only reads the latest N rows.

**Future:**
- Cap rows per task and/or age (e.g. keep last 100, or 30 days)
- Prune on write or on a periodic maintenance pass

**Acceptance:**
- Long-lived installs do not grow the log unbounded

### Expand automated tests

**Status:** `planned`  
**Today:** Unit coverage is mostly validation, schedule math, URL close needle, and Linux browser-id parsing. Executor, scheduler, repository, and frontend are largely untested.

**Future:**
- Repository / migration tests (status transitions, pause/resume, reschedule rules)
- Executor tests for success, soft close miss, and hard failure paths
- Lightweight frontend tests for task status controls where practical

**Acceptance:**
- Regressions in schedule update and pause/retry are caught by `cargo test` (and any added frontend suite)

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

### Compact task card UI

**Status:** `planned`  
**Today:** Task cards show full dual-timezone lines, schedule TZ, last run, errors, and a tall action column — readable but spacious, especially with many tasks.

**Future:**
- Denser card layout (tighter spacing, secondary times/meta less prominent)
- Keep local + schedule-zone times and last-error/log access without losing scannability
- Optionally collapse secondary details until expanded

**Acceptance:**
- More tasks fit on screen without scrolling as much
- Dual-timezone clarity and Log / Activity access remain obvious

### 12 / 24-hour time format setting

**Status:** `planned`  
**Today:** Times use locale-default formatting (often 12-hour via `date-fns` `PPp` / `toLocaleString`) with no user override.

**Future:**
- Settings toggle: 12-hour vs 24-hour clock
- Persist in `settings` and apply across task cards, Activity, forms/tooltips where times are shown
- Keep schedule-zone dual display consistent with the chosen format

**Acceptance:**
- Choosing 24-hour shows times like `15:30` everywhere the preference applies
- Preference survives app restart

### Calendar schedule view

**Status:** `planned`  
**Today:** Schedules are only a vertical task list; no calendar overview of upcoming opens/closes.

**Future:**
- Calendar (day/week/month) showing scheduled open/close events
- Distinct colors per task (or per action: open vs close) so overlapping sessions are scannable
- Click through to the task / edit flow

**Acceptance:**
- User can see multiple tasks’ next runs on a calendar without reading every card
- Color coding remains readable in light theme (and dark when that ships)

### React router (leave modals behind)

**Status:** `planned`  
**Today:** Almost everything lives on one screen; Settings and Activity are modals. As features grow (calendar, profiles, denser editing), modals will not scale.

**Future:**
- Add a React router (e.g. React Router) with real routes: tasks list, task create/edit, activity, settings, calendar
- Prefer full pages / nested layouts over stacking modals
- Keep deep-linkable paths where useful inside the desktop shell

**Acceptance:**
- Primary flows (create/edit, activity, settings, calendar) are navigable routes, not only overlays
- Back/forward or in-app nav works without losing task list context awkwardly

### Docs / capabilities polish

**Status:** `planned`  
**Today:** Some README wording still reads like a web app (“server-side validation”); Linux close dependencies are under-documented; DB troubleshooting defaults to “delete the DB.”

**Future:**
- Document Linux close tools and Wayland limits
- Clarify desktop IPC validation wording
- Verify notification / autostart plugin permissions in Tauri capabilities so soft-fail paths are intentional, not silent misconfig

**Acceptance:**
- README troubleshooting matches real failure modes users hit on each OS

---

## Notes

- Prefer updating this file when scope or status changes, rather than re-advertising unfinished items as Features in the README.
- Related analysis: intention-vs-implementation review (local Cursor canvas).
- **Close (Win/Linux tab):** remains `on-hold` — no clear resolve yet (CDP / extension / native messaging still undecided). Ship safer scoping and soft close-miss handling when useful; do not promise true tab close until a path is chosen.
- **Next iteration priority (suggested):** reschedule-on-repeat-edit → tests → UI polish (compact cards, 12/24h, calendar, React router). Harden close scoping only where safe without claiming tab control. Profile management / dark mode remain polish.
