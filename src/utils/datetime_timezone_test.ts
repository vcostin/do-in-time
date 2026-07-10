import {
  utcToZonedDatetimeString,
  zonedDatetimeStringToUtc,
} from './datetime.ts';
import {
  abbrToIana,
  extractTimezoneAbbr,
  resolveScheduleTimezone,
} from './timezone.ts';

function assertEquals(actual: unknown, expected: unknown, msg?: string) {
  if (actual !== expected) {
    throw new Error(
      msg ?? `assertEquals failed: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
    );
  }
}

Deno.test('abbrToIana maps common abbreviations', () => {
  assertEquals(abbrToIana('ET'), 'America/New_York');
  assertEquals(abbrToIana('pt'), 'America/Los_Angeles');
  assertEquals(abbrToIana('JST'), 'Asia/Tokyo');
  assertEquals(abbrToIana('CET'), 'Europe/Paris');
  assertEquals(abbrToIana('UTC'), 'UTC');
  assertEquals(abbrToIana('NOPE'), null);
});

Deno.test('extractTimezoneAbbr finds rightmost known token', () => {
  assertEquals(extractTimezoneAbbr('January 31st from 9am to 11am ET'), 'ET');
  assertEquals(extractTimezoneAbbr('tomorrow at 2pm PT'), 'PT');
  assertEquals(extractTimezoneAbbr('March 15 9:00 JST'), 'JST');
  assertEquals(extractTimezoneAbbr('tomorrow at 2pm'), null);
});

Deno.test('resolveScheduleTimezone uses abbr or fallback', () => {
  assertEquals(
    resolveScheduleTimezone({ abbr: 'ET', fallbackIana: 'UTC' }).iana,
    'America/New_York',
  );
  assertEquals(
    resolveScheduleTimezone({ abbr: null, fallbackIana: 'Europe/Berlin' }).iana,
    'Europe/Berlin',
  );
  const bad = resolveScheduleTimezone({ abbr: 'ZZZ', fallbackIana: 'UTC' });
  assertEquals(bad.iana, 'UTC');
  assertEquals(typeof bad.error, 'string');
});

Deno.test('9am America/New_York round-trips to known UTC (standard time)', () => {
  const wall = '2026-01-31T09:00';
  const utc = zonedDatetimeStringToUtc(wall, 'America/New_York');
  assertEquals(utc, '2026-01-31T14:00:00.000Z');
  assertEquals(utcToZonedDatetimeString(utc, 'America/New_York'), wall);
});

Deno.test('9am America/New_York round-trips across DST (EDT)', () => {
  const wall = '2026-07-15T09:00';
  const utc = zonedDatetimeStringToUtc(wall, 'America/New_York');
  assertEquals(utc, '2026-07-15T13:00:00.000Z');
  assertEquals(utcToZonedDatetimeString(utc, 'America/New_York'), wall);
});

Deno.test('JST wall clock converts to UTC', () => {
  const wall = '2026-03-15T09:00';
  const utc = zonedDatetimeStringToUtc(wall, 'Asia/Tokyo');
  assertEquals(utc, '2026-03-15T00:00:00.000Z');
});
