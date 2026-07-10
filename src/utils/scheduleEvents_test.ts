import {
  addOneInterval,
  expandScheduleEvents,
  startOfLocalDay,
  startOfLocalWeek,
  type ScheduleTaskInput,
} from './scheduleEvents.ts';

function assertEquals(actual: unknown, expected: unknown, msg?: string) {
  if (actual !== expected) {
    throw new Error(
      msg ?? `assertEquals failed: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
    );
  }
}

function assert(cond: boolean, msg: string) {
  if (!cond) {
    throw new Error(msg);
  }
}

Deno.test('addOneInterval daily in UTC', () => {
  const next = addOneInterval('2026-01-01T09:00:00.000Z', 'daily', 'UTC');
  assertEquals(next.toISOString(), '2026-01-02T09:00:00.000Z');
});

Deno.test('addOneInterval weekly in UTC', () => {
  const next = addOneInterval('2026-01-01T09:00:00.000Z', 'weekly', 'UTC');
  assertEquals(next.toISOString(), '2026-01-08T09:00:00.000Z');
});

Deno.test('addOneInterval monthly clamps day', () => {
  // Jan 31 → Feb 28 in 2026
  const next = addOneInterval('2026-01-31T15:00:00.000Z', 'monthly', 'UTC');
  assertEquals(next.toISOString(), '2026-02-28T15:00:00.000Z');
});

Deno.test('addOneInterval daily preserves America/New_York wall clock across DST', () => {
  // 2026-03-07 09:00 EST → 2026-03-08 09:00 EDT (UTC offset changes)
  const start = '2026-03-07T14:00:00.000Z'; // 09:00 EST
  const next = addOneInterval(start, 'daily', 'America/New_York');
  assertEquals(next.toISOString(), '2026-03-08T13:00:00.000Z'); // 09:00 EDT
});

function dailyTask(overrides: Partial<ScheduleTaskInput> = {}): ScheduleTaskInput {
  return {
    id: 1,
    name: 'Standup',
    status: 'active',
    start_time: '2026-07-10T13:00:00.000Z',
    close_time: '2026-07-10T15:00:00.000Z',
    timezone: 'UTC',
    execution_count: 0,
    next_open_execution: '2026-07-10T13:00:00.000Z',
    next_close_execution: '2026-07-10T15:00:00.000Z',
    repeat_config: {
      interval: 'daily',
      end_after: null,
      end_date: null,
    },
    ...overrides,
  };
}

Deno.test('expandScheduleEvents expands daily opens and closes across a week', () => {
  const rangeStart = new Date('2026-07-10T00:00:00.000Z');
  const rangeEnd = new Date('2026-07-13T00:00:00.000Z');
  const events = expandScheduleEvents([dailyTask()], rangeStart, rangeEnd);
  const opens = events.filter((e) => e.action === 'open');
  const closes = events.filter((e) => e.action === 'close');
  assertEquals(opens.length, 3);
  assertEquals(closes.length, 3);
  assertEquals(opens[0].atIso, '2026-07-10T13:00:00.000Z');
  assertEquals(opens[2].atIso, '2026-07-12T13:00:00.000Z');
  assertEquals(closes[0].atIso, '2026-07-10T15:00:00.000Z');
});

Deno.test('expandScheduleEvents respects end_after remaining opens', () => {
  const rangeStart = new Date('2026-07-10T00:00:00.000Z');
  const rangeEnd = new Date('2026-07-20T00:00:00.000Z');
  const events = expandScheduleEvents(
    [
      dailyTask({
        execution_count: 1,
        repeat_config: { interval: 'daily', end_after: 3, end_date: null },
      }),
    ],
    rangeStart,
    rangeEnd,
  );
  const opens = events.filter((e) => e.action === 'open');
  // end_after 3, already executed 1 → 2 opens left
  assertEquals(opens.length, 2);
});

Deno.test('expandScheduleEvents one-shot only emits next_*', () => {
  const rangeStart = new Date('2026-07-01T00:00:00.000Z');
  const rangeEnd = new Date('2026-07-31T00:00:00.000Z');
  const events = expandScheduleEvents(
    [
      dailyTask({
        repeat_config: null,
        next_open_execution: '2026-07-15T10:00:00.000Z',
        next_close_execution: '2026-07-15T12:00:00.000Z',
      }),
    ],
    rangeStart,
    rangeEnd,
  );
  assertEquals(events.length, 2);
  assertEquals(events[0].action, 'open');
  assertEquals(events[1].action, 'close');
});

Deno.test('expandScheduleEvents skips non-active tasks', () => {
  const rangeStart = new Date('2026-07-10T00:00:00.000Z');
  const rangeEnd = new Date('2026-07-12T00:00:00.000Z');
  const events = expandScheduleEvents(
    [dailyTask({ status: 'disabled' })],
    rangeStart,
    rangeEnd,
  );
  assertEquals(events.length, 0);
});

Deno.test('startOfLocalWeek returns Monday', () => {
  // Wednesday Jul 8, 2026 local — construct via Y/M/D to avoid TZ ISO ambiguity
  const wed = new Date(2026, 6, 8, 15, 0, 0);
  const monday = startOfLocalWeek(wed);
  assertEquals(monday.getDay(), 1);
  assertEquals(monday.getDate(), 6);
  assertEquals(startOfLocalDay(monday).getTime(), monday.getTime());
});

Deno.test('expand includes close when open is before range', () => {
  const rangeStart = new Date('2026-07-11T00:00:00.000Z');
  const rangeEnd = new Date('2026-07-12T00:00:00.000Z');
  const events = expandScheduleEvents(
    [
      dailyTask({
        start_time: '2026-07-10T22:00:00.000Z',
        close_time: '2026-07-11T02:00:00.000Z',
        next_open_execution: '2026-07-10T22:00:00.000Z',
        next_close_execution: '2026-07-11T02:00:00.000Z',
        repeat_config: null,
      }),
    ],
    rangeStart,
    rangeEnd,
  );
  // One-shot path only emits next_* — close is in range
  assertEquals(events.length, 1);
  assertEquals(events[0].action, 'close');
});

Deno.test('expand repeating close spills into next day', () => {
  const rangeStart = new Date('2026-07-11T00:00:00.000Z');
  const rangeEnd = new Date('2026-07-12T00:00:00.000Z');
  const events = expandScheduleEvents(
    [
      dailyTask({
        start_time: '2026-07-10T22:00:00.000Z',
        close_time: '2026-07-11T02:00:00.000Z',
        next_open_execution: '2026-07-10T22:00:00.000Z',
        next_close_execution: '2026-07-11T02:00:00.000Z',
        repeat_config: { interval: 'daily', end_after: null, end_date: null },
      }),
    ],
    rangeStart,
    rangeEnd,
  );
  const closes = events.filter((e) => e.action === 'close');
  assert(closes.some((e) => e.atIso === '2026-07-11T02:00:00.000Z'), 'expected spillover close');
});

Deno.test('expand sorts by time', () => {
  const rangeStart = new Date('2026-07-10T00:00:00.000Z');
  const rangeEnd = new Date('2026-07-11T00:00:00.000Z');
  const events = expandScheduleEvents(
    [
      dailyTask({
        id: 2,
        name: 'Late',
        next_open_execution: '2026-07-10T18:00:00.000Z',
        next_close_execution: null,
        close_time: null,
        repeat_config: null,
      }),
      dailyTask({
        id: 1,
        name: 'Early',
        next_open_execution: '2026-07-10T08:00:00.000Z',
        next_close_execution: null,
        close_time: null,
        repeat_config: null,
      }),
    ],
    rangeStart,
    rangeEnd,
  );
  assert(events.length === 2, 'expected 2 events');
  assertEquals(events[0].taskName, 'Early');
  assertEquals(events[1].taskName, 'Late');
});
