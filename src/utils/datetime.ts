import { formatInTimeZone, fromZonedTime } from 'date-fns-tz';

function ensureSeconds(wall: string): string {
  if (/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(wall)) {
    return `${wall}:00`;
  }
  return wall;
}

function pad(n: number, width: number): string {
  return String(n).padStart(width, '0');
}

/** Advance a wall `YYYY-MM-DDTHH:mm:ss` by one minute (calendar-naive). */
function addOneMinuteToWall(wall: string): string {
  const [datePart, timePart = '00:00:00'] = wall.split('T');
  const [y, m, d] = datePart.split('-').map(Number);
  const [hh, mm, ss] = timePart.split(':').map(Number);
  const utc = Date.UTC(y, m - 1, d, hh, mm, ss || 0);
  const next = new Date(utc + 60_000);
  return `${pad(next.getUTCFullYear(), 4)}-${pad(next.getUTCMonth() + 1, 2)}-${pad(next.getUTCDate(), 2)}T${pad(next.getUTCHours(), 2)}:${pad(next.getUTCMinutes(), 2)}:${pad(next.getUTCSeconds(), 2)}`;
}

/**
 * Convert a schedule-zone wall clock to a UTC Date.
 * Round-trips through the zone so DST gaps are detected.
 * When `snapGap` is true, advances minute-by-minute to the next valid local time
 * (mirrors Rust `at_wall_clock_on`).
 */
export function wallToUtc(
  wall: string,
  timeZone: string,
  options: { snapGap?: boolean } = {},
): Date {
  let candidate = ensureSeconds(wall);
  for (let i = 0; i < 180; i++) {
    const utc = fromZonedTime(candidate, timeZone);
    const back = formatInTimeZone(utc, timeZone, "yyyy-MM-dd'T'HH:mm:ss");
    if (back === candidate) {
      return utc;
    }
    if (!options.snapGap) {
      throw new Error(
        `Nonexistent local time (DST gap): ${wall} in ${timeZone}`,
      );
    }
    candidate = addOneMinuteToWall(candidate);
  }
  throw new Error(`Could not resolve local time near ${wall} in ${timeZone}`);
}

/** Wall-clock datetime-local string in an IANA zone from a UTC instant. */
export function utcToZonedDatetimeString(utcIsoString: string, timeZone: string): string {
  return formatInTimeZone(new Date(utcIsoString), timeZone, "yyyy-MM-dd'T'HH:mm");
}

/** Interpret a datetime-local wall clock in an IANA zone as a UTC ISO string. */
export function zonedDatetimeStringToUtc(localDatetimeString: string, timeZone: string): string {
  return wallToUtc(localDatetimeString, timeZone, { snapGap: false }).toISOString();
}

/**
 * Build a Date whose local Y/M/D/H/M/S match "now" in `timeZone`.
 * Used as chrono-node's reference so relative phrases like "tomorrow" follow
 * the schedule zone's calendar, not the machine's.
 */
export function referenceDateInZone(timeZone: string, instant: Date = new Date()): Date {
  const wall = formatInTimeZone(instant, timeZone, "yyyy-MM-dd'T'HH:mm:ss");
  const [datePart, timePart = '00:00:00'] = wall.split('T');
  const [y, m, d] = datePart.split('-').map(Number);
  const [hh, mm, ss] = timePart.split(':').map(Number);
  return new Date(y, m - 1, d, hh, mm, ss || 0);
}

/** Formats a UTC ISO string for display in the operator's local timezone. */
export function formatUtcForDisplay(utcIsoString: string, use24Hour = true): string {
  return formatInTimeZone(
    new Date(utcIsoString),
    getLocalTimeZone(),
    displayDateTimePattern(use24Hour),
  );
}

/** Formats a UTC ISO string as a readable wall time in an IANA zone. */
export function formatUtcInZone(
  utcIsoString: string,
  timeZone: string,
  use24Hour = true,
): string {
  return formatInTimeZone(new Date(utcIsoString), timeZone, displayDateTimePattern(use24Hour));
}

/** date-fns pattern for readable date+time in local or schedule zone. */
export function displayDateTimePattern(use24Hour: boolean): string {
  return use24Hour ? 'MMM d, yyyy HH:mm' : 'MMM d, yyyy h:mm a';
}

/** Formats for react-datepicker date+time field (display only; storage stays 24h). */
export function pickerDateTimeFormats(use24Hour: boolean): {
  dateFormat: string;
  timeFormat: string;
  placeholder: string;
} {
  if (use24Hour) {
    return {
      dateFormat: 'yyyy-MM-dd HH:mm',
      timeFormat: 'HH:mm',
      placeholder: 'YYYY-MM-DD HH:mm',
    };
  }
  return {
    dateFormat: 'yyyy-MM-dd h:mm aa',
    timeFormat: 'h:mm aa',
    placeholder: 'YYYY-MM-DD h:mm aa',
  };
}

function getLocalTimeZone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
  } catch {
    return 'UTC';
  }
}

/** Split a datetime-local wall string into date and HH:mm parts. */
export function splitDatetimeLocal(value: string): { date: string; time: string } {
  if (!value) {
    return { date: '', time: '' };
  }
  const [date = '', rest = ''] = value.split('T');
  return { date, time: rest.slice(0, 5) };
}

/** Normalize typed calendar date to YYYY-MM-DD when valid. */
export function normalizeDate(date: string): string | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date.trim());
  if (!match) {
    return null;
  }
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  if (month < 1 || month > 12 || day < 1 || day > 31) {
    return null;
  }
  const parsed = new Date(Date.UTC(year, month - 1, day));
  if (
    parsed.getUTCFullYear() !== year ||
    parsed.getUTCMonth() !== month - 1 ||
    parsed.getUTCDate() !== day
  ) {
    return null;
  }
  return `${match[1]}-${match[2]}-${match[3]}`;
}

/** Normalize typed time to HH:mm when valid (accepts H:mm or HH:mm). */
export function normalizeTime(time: string): string | null {
  const match = /^(\d{1,2}):(\d{2})$/.exec(time.trim());
  if (!match) {
    return null;
  }
  const hours = Number(match[1]);
  const minutes = Number(match[2]);
  if (hours < 0 || hours > 23 || minutes < 0 || minutes > 59) {
    return null;
  }
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
}

/**
 * Compose date + time into YYYY-MM-DDTHH:mm.
 * Returns '' when both empty; null when incomplete or invalid.
 */
export function composeDatetimeLocal(date: string, time: string): string | null {
  if (!date.trim() && !time.trim()) {
    return '';
  }
  const normalizedDate = normalizeDate(date);
  const normalizedTime = normalizeTime(time);
  if (!normalizedDate || !normalizedTime) {
    return null;
  }
  return `${normalizedDate}T${normalizedTime}`;
}
