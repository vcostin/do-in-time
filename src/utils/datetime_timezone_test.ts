import {
  composeDatetimeLocal,
  displayDateTimePattern,
  normalizeDate,
  normalizeTime,
  pickerDateTimeFormats,
  referenceDateInZone,
  splitDatetimeLocal,
  utcToZonedDatetimeString,
  wallToUtc,
  zonedDatetimeStringToUtc,
} from './datetime.ts';
import {
  abbrToIana,
  extractTimezoneAbbr,
  hasNumericUtcOffset,
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

Deno.test('extractTimezoneAbbr ignores GMT/UTC when numeric offset present', () => {
  assertEquals(extractTimezoneAbbr('March 15 at 2pm GMT+5'), null);
  assertEquals(extractTimezoneAbbr('tomorrow 9am UTC-3'), null);
  assertEquals(hasNumericUtcOffset('2pm GMT+5'), true);
  assertEquals(hasNumericUtcOffset('2pm ET'), false);
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

Deno.test('referenceDateInZone mirrors wall clock in target zone', () => {
  // 2026-07-10 22:30 UTC = 2026-07-11 07:30 JST
  const instant = new Date('2026-07-10T22:30:00.000Z');
  const ref = referenceDateInZone('Asia/Tokyo', instant);
  assertEquals(ref.getFullYear(), 2026);
  assertEquals(ref.getMonth(), 6); // July
  assertEquals(ref.getDate(), 11);
  assertEquals(ref.getHours(), 7);
  assertEquals(ref.getMinutes(), 30);
});

Deno.test('zonedDatetimeStringToUtc rejects America/New_York spring gap', () => {
  let threw = false;
  try {
    zonedDatetimeStringToUtc('2026-03-08T02:30', 'America/New_York');
  } catch {
    threw = true;
  }
  assertEquals(threw, true);
});

Deno.test('wallToUtc snapGap advances past America/New_York spring gap', () => {
  const utc = wallToUtc('2026-03-08T02:30:00', 'America/New_York', { snapGap: true });
  // First valid local after gap is 03:00 EDT = 07:00Z
  assertEquals(utc.toISOString(), '2026-03-08T07:00:00.000Z');
});

Deno.test('splitDatetimeLocal and composeDatetimeLocal round-trip', () => {
  assertEquals(splitDatetimeLocal('').date, '');
  assertEquals(splitDatetimeLocal('2026-07-10T14:30').time, '14:30');
  assertEquals(composeDatetimeLocal('2026-07-10', '14:30'), '2026-07-10T14:30');
  assertEquals(composeDatetimeLocal('2026-07-10', '9:05'), '2026-07-10T09:05');
  assertEquals(composeDatetimeLocal('', ''), '');
  assertEquals(composeDatetimeLocal('2026-07-10', ''), null);
  assertEquals(composeDatetimeLocal('', '14:30'), null);
  assertEquals(composeDatetimeLocal('2026-02-30', '09:00'), null);
});

Deno.test('normalizeTime pads and rejects invalid', () => {
  assertEquals(normalizeTime('9:30'), '09:30');
  assertEquals(normalizeTime('23:59'), '23:59');
  assertEquals(normalizeTime('24:00'), null);
  assertEquals(normalizeTime('12:60'), null);
  assertEquals(normalizeTime('noon'), null);
});

Deno.test('normalizeDate accepts calendar dates only', () => {
  assertEquals(normalizeDate('2026-07-10'), '2026-07-10');
  assertEquals(normalizeDate('2026-02-30'), null);
  assertEquals(normalizeDate('07/10/2026'), null);
  assertEquals(normalizeDate('not-a-date'), null);
});

Deno.test('picker and display formats follow 12/24h preference', () => {
  assertEquals(pickerDateTimeFormats(true).timeFormat, 'HH:mm');
  assertEquals(pickerDateTimeFormats(false).timeFormat, 'h:mm aa');
  assertEquals(displayDateTimePattern(true).includes('HH:mm'), true);
  assertEquals(displayDateTimePattern(false).includes('h:mm a'), true);
});
