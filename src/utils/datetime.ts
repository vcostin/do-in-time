// Utility functions for handling UTC/Local time conversions

/**
 * Converts a UTC ISO string to local datetime-local format (YYYY-MM-DDTHH:mm)
 * Used for populating datetime-local inputs when editing tasks
 */
export function utcToLocalDatetimeString(utcIsoString: string): string {
  const date = new Date(utcIsoString);
  // Get local time components
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');

  return `${year}-${month}-${day}T${hours}:${minutes}`;
}

/**
 * Converts a local datetime-local string to UTC ISO string
 * Used when submitting forms to the backend
 */
export function localDatetimeStringToUtc(localDatetimeString: string): string {
  // new Date() interprets datetime-local format as local time
  return new Date(localDatetimeString).toISOString();
}

/**
 * Formats a UTC ISO string for display in local time
 * Note: date-fns format() already handles this automatically
 */
export function formatUtcForDisplay(utcIsoString: string): string {
  return new Date(utcIsoString).toLocaleString();
}
