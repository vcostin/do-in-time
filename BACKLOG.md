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

---

## Notes

- Prefer updating this file when scope or status changes, rather than re-advertising unfinished items as Features in the README.
- Related analysis: intention-vs-implementation review (local Cursor canvas).
