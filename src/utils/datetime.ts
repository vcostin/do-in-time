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
export function formatUtcForDisplay(utcIsoString: string): string {
  return new Date(utcIsoString).toLocaleString();
}

/** Formats a UTC ISO string as a readable wall time in an IANA zone. */
export function formatUtcInZone(utcIsoString: string, timeZone: string): string {
  return formatInTimeZone(new Date(utcIsoString), timeZone, 'PPp');
}
