import { formatInTimeZone, fromZonedTime } from 'date-fns-tz';

/**
 * Converts a UTC ISO string to machine-local datetime-local format (YYYY-MM-DDTHH:mm).
 * Prefer utcToZonedDatetimeString when the schedule timezone is known.
 */
export function utcToLocalDatetimeString(utcIsoString: string): string {
  const date = new Date(utcIsoString);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');

  return `${year}-${month}-${day}T${hours}:${minutes}`;
}

/**
 * Converts a machine-local datetime-local string to UTC ISO.
 * Prefer zonedDatetimeStringToUtc when the schedule timezone is known.
 */
export function localDatetimeStringToUtc(localDatetimeString: string): string {
  return new Date(localDatetimeString).toISOString();
}

/** Wall-clock datetime-local string in an IANA zone from a UTC instant. */
export function utcToZonedDatetimeString(utcIsoString: string, timeZone: string): string {
  return formatInTimeZone(new Date(utcIsoString), timeZone, "yyyy-MM-dd'T'HH:mm");
}

/** Interpret a datetime-local wall clock in an IANA zone as a UTC ISO string. */
export function zonedDatetimeStringToUtc(localDatetimeString: string, timeZone: string): string {
  return fromZonedTime(localDatetimeString, timeZone).toISOString();
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
