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

### Reschedule when repeat config changes

**Status:** `done`  
- `update_task` re-derives `next_*` when `repeat_config` is added, removed, or edited (same path as time edits)
- Close offset recomputed with the new plan

### Expand automated tests (executor + schedule status)

**Status:** `done`  
- Executor outcome helpers: soft close miss, hard failure, success schedule / status
- Schedule status: past one-shots, remove/add repeat, future-close-only stays Active
- Broader suite (repository integration, frontend) remains optional follow-up

### 12 / 24-hour time format setting

**Status:** `done`  
- Settings toggle `use_24_hour_clock` (default 24h)
- Shared format helpers drive task cards, Activity, and react-datepicker
- Stored wall times remain `YYYY-MM-DDTHH:mm` (24h) regardless of display

### Compact task card UI

**Status:** `done`  
- Denser cards: tighter padding, smaller type, horizontal action chips
- Primary row shows next open/close when Active (else start/close); dual-timezone secondary lines kept
- Browser / repeat / URL on one meta line; start/close + last run + schedule TZ under **More**
- Per-task **Log** and header **Activity** unchanged

### Calendar schedule view

**Status:** `done`  
- List | Calendar toggle; Day / Week / Month navigation
- Expands repeating tasks client-side (same interval/DST rules as the scheduler) into open/close events
- Per-task colors; open = solid chip, close = dashed; click event → edit task
- Month cells show task dots and drill into Day

### React router (leave modals behind)

**Status:** `done`  
- `react-router-dom` with `HashRouter` (Tauri-friendly deep links)
- Routes: `/`, `/calendar`, `/tasks/new`, `/tasks/:id/edit`, `/activity`, `/settings`
- Shared `AppLayout`; Settings/Activity are pages (modals removed); tray `open-settings` navigates to `/settings`
- `TasksProvider` shares task state across list/calendar/edit

### Docs / capabilities polish

**Status:** `done`  
- README Features catch-up (calendar, routes, 12/24h, unsaved guard, soft close, settings)
- “Server-side” wording → Rust / Tauri IPC validation
- Linux close tools (`wmctrl` / `xdotool`), Wayland limits, soft close-miss troubleshooting
- DB wipe documented as last resort (prefer Log / Activity)
- Capabilities grant `notification:default` and `autostart:default`; soft-fail when OS denies remains intentional

---

## Planned

### Re-enable Linux AppImage bundling

**Status:** `planned`  
**Today:** `bundle.targets` is `["deb", "rpm"]` only. AppImage used to build, then started failing on Arch after **gdk-pixbuf 2.44+** switched to **glycin** and stopped shipping `/usr/lib/gdk-pixbuf-2.0/2.10.0`. Tauri’s `linuxdeploy-plugin-gtk` still `cp`s that tree and exits. (`fuse2` is unrelated — it only affects mounting AppImages; Tauri already sets `APPIMAGE_EXTRACT_AND_RUN`.)

**Check later (upstream fixed?):**
- Does `deno task tauri build -- --bundles appimage` succeed on current Arch without patching `~/.cache/tauri/linuxdeploy-plugin-gtk.sh`?
- Relevant pieces: [linuxdeploy-plugin-gtk](https://github.com/linuxdeploy/linuxdeploy-plugin-gtk) / [tauri fork](https://github.com/tauri-apps/linuxdeploy-plugin-gtk), gdk-pixbuf ≥ 2.44 + glycin, Tauri bundler AppImage path

**When unblocked:**
- Restore AppImage in `bundle.targets` (e.g. `"all"` or add `"appimage"`)
- Drop the README note that AppImage is omitted for this reason

**Acceptance:**
- Default `tauri build` produces a working AppImage on Arch without local plugin hacks
- `.deb` / `.rpm` keep working

### Harden Linux / Windows close scoping

**Status:** `planned`  
**Today:** Linux title-close ignores which browser was requested; `pkill -f` uses short names (`chrome`, `firefox`) that can match unrelated processes. Windows `taskkill` is process-image-wide when `allow_close_all` is set.

**Future:**
- Scope window matching to the task’s browser where possible
- Prefer exact process-name kills (`pkill -x` / equivalent) over substring cmdline matches
- Surface clearer in-app errors when close tools are missing (README already documents `wmctrl` / `xdotool` and Wayland limits)

**Acceptance:**
- Close/kill paths do not intentionally target other browsers or unrelated processes
- Missing tools / unsupported environments surface a clear error, not a silent no-op

### Execution log retention

**Status:** `planned`  
**Today:** `task_execution_log` is append-only with no prune; UI only reads the latest N rows.

**Future:**
- Cap rows per task and/or age (e.g. keep last 100, or 30 days)
- Prune on write or on a periodic maintenance pass

**Acceptance:**
- Long-lived installs do not grow the log unbounded

### Profile management

**Status:** `planned`  
**Today:** Backend still accepts `browser_profile` (launch args + validation), but the task form field is **hidden** so users are not offered an unfinished free-text control. Existing stored profiles are preserved on edit.

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

### Nuke / hard-reset database

**Status:** `planned`  
**Priority:** very low — nice-to-have; may not be worth shipping  
**Today:** Resetting means quitting the app and deleting `data.db` / `dev-data.db` by hand (see README).

**Idea (if ever):**
- Settings control: “Reset all data” / nuke DB with a strong confirmation (type-to-confirm or similar)
- Drop or recreate the SQLite file, re-run schema init, refresh UI to empty state
- Possibly offer “reset this build’s DB only” so release and `dev-data.db` stay independent

**Open question:** Whether an in-app nuke is safer than documenting file delete, or just a footgun for accidental wipes.

**Acceptance (if built):**
- User can wipe tasks/logs/settings for the current DB without hunting the filesystem
- Accidental reset is hard; release vs debug DB are not confused

---

## Notes

- Prefer updating this file when scope or status changes, rather than re-advertising unfinished items as Features in the README.
- Related analysis: intention-vs-implementation review (local Cursor canvas).
- **Close (Win/Linux tab):** remains `on-hold` — no clear resolve yet (CDP / extension / native messaging still undecided). Ship safer scoping and soft close-miss handling when useful; do not promise true tab close until a path is chosen.
- **Next iteration priority (suggested):** Harden close scoping only where safe without claiming tab control. Profile management / dark mode remain polish. Optional: repository/frontend tests; re-check AppImage on Arch. **Nuke DB** is backlog-only, very low priority.
