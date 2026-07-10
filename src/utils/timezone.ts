/**
 * Map common timezone abbreviations (as recognized by chrono-node) to canonical
 * IANA zones for DST-safe repeat scheduling. Ambiguous abbrs use the US/common
 * convention that matches chrono-node's offset table where possible.
 */
export const TIMEZONE_ABBR_TO_IANA: Record<string, string> = {
  UTC: 'UTC',
  GMT: 'UTC',
  Z: 'UTC',

  // North America
  ET: 'America/New_York',
  EST: 'America/New_York',
  EDT: 'America/New_York',
  CT: 'America/Chicago',
  CST: 'America/Chicago',
  CDT: 'America/Chicago',
  MT: 'America/Denver',
  MST: 'America/Denver',
  MDT: 'America/Denver',
  PT: 'America/Los_Angeles',
  PST: 'America/Los_Angeles',
  PDT: 'America/Los_Angeles',
  AKST: 'America/Anchorage',
  AKDT: 'America/Anchorage',
  HAST: 'Pacific/Honolulu',
  HADT: 'Pacific/Honolulu',
  AST: 'America/Halifax',
  ADT: 'America/Halifax',
  NST: 'America/St_Johns',
  NDT: 'America/St_Johns',

  // Europe
  WET: 'Europe/Lisbon',
  WEST: 'Europe/Lisbon',
  CET: 'Europe/Paris',
  CEST: 'Europe/Paris',
  EET: 'Europe/Bucharest',
  EEST: 'Europe/Bucharest',
  BST: 'Europe/London',
  MSK: 'Europe/Moscow',

  // Asia / Pacific
  IST: 'Asia/Kolkata',
  JST: 'Asia/Tokyo',
  KST: 'Asia/Seoul',
  HKT: 'Asia/Hong_Kong',
  SGT: 'Asia/Singapore',
  AWST: 'Australia/Perth',
  ACST: 'Australia/Adelaide',
  ACDT: 'Australia/Adelaide',
  AEST: 'Australia/Sydney',
  AEDT: 'Australia/Sydney',
  NZST: 'Pacific/Auckland',
  NZDT: 'Pacific/Auckland',

  // Middle East / Africa
  IDT: 'Asia/Jerusalem',
  IRST: 'Asia/Tehran',
  IRDT: 'Asia/Tehran',
  SAST: 'Africa/Johannesburg',
  CAT: 'Africa/Harare',
  EAT: 'Africa/Nairobi',
};

/** Curated IANA zones for the schedule-timezone picker. */
export const COMMON_IANA_TIMEZONES: string[] = [
  'UTC',
  'America/New_York',
  'America/Chicago',
  'America/Denver',
  'America/Los_Angeles',
  'America/Toronto',
  'America/Sao_Paulo',
  'Europe/London',
  'Europe/Paris',
  'Europe/Berlin',
  'Europe/Bucharest',
  'Europe/Moscow',
  'Africa/Cairo',
  'Asia/Dubai',
  'Asia/Kolkata',
  'Asia/Singapore',
  'Asia/Shanghai',
  'Asia/Tokyo',
  'Asia/Seoul',
  'Australia/Sydney',
  'Pacific/Auckland',
];

const ABBR_TOKEN_RE = /\b([A-Za-z]{2,5})\b/g;

export function getSystemTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
}

/** Extract the rightmost known timezone abbreviation from free text. */
export function extractTimezoneAbbr(text: string): string | null {
  const matches = [...text.matchAll(ABBR_TOKEN_RE)].map((m) => m[1].toUpperCase());
  for (let i = matches.length - 1; i >= 0; i--) {
    const token = matches[i];
    if (TIMEZONE_ABBR_TO_IANA[token]) {
      return token;
    }
  }
  return null;
}

export function abbrToIana(abbr: string): string | null {
  return TIMEZONE_ABBR_TO_IANA[abbr.toUpperCase()] ?? null;
}

export function resolveScheduleTimezone(options: {
  abbr?: string | null;
  fallbackIana: string;
}): { iana: string; error?: string } {
  const { abbr, fallbackIana } = options;
  if (!abbr) {
    return { iana: fallbackIana };
  }
  const iana = abbrToIana(abbr);
  if (!iana) {
    return {
      iana: fallbackIana,
      error: `Unrecognized timezone abbreviation "${abbr}". Use an IANA zone from the picker, or a known abbr like ET, PT, JST.`,
    };
  }
  return { iana };
}

/** Zones for a <select>, ensuring system + current task zone are present. */
export function scheduleTimezoneOptions(current?: string | null): string[] {
  const system = getSystemTimeZone();
  const set = new Set<string>([system, ...COMMON_IANA_TIMEZONES]);
  if (current) {
    set.add(current);
  }
  return [...set].sort((a, b) => {
    if (a === system) return -1;
    if (b === system) return 1;
    return a.localeCompare(b);
  });
}
