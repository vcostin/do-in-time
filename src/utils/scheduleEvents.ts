import { formatInTimeZone } from 'date-fns-tz';
import { wallToUtc } from './datetime';

export type ScheduleAction = 'open' | 'close';
export type ScheduleInterval = 'daily' | 'weekly' | 'monthly';

/** Minimal task shape needed to expand calendar events (satisfied by `Task`). */
export interface ScheduleTaskInput {
  id?: number | null;
  name: string;
  status: string;
  start_time: string;
  close_time?: string | null;
  timezone: string;
  execution_count: number;
  next_open_execution?: string | null;
  next_close_execution?: string | null;
  repeat_config?: {
    interval: ScheduleInterval | string;
    end_after?: number | null;
    end_date?: string | null;
  } | null;
}

export interface ScheduleEvent {
  taskId: number;
  taskName: string;
  action: ScheduleAction;
  at: Date;
  atIso: string;
}

/** Stable pastel palette for light theme (and readable dark: variants in UI). */
const TASK_COLOR_COUNT = 8;

export function taskColorIndex(taskId: number): number {
  return Math.abs(taskId) % TASK_COLOR_COUNT;
}

/** Tailwind class sets keyed by color index — open uses solid, close uses outline feel in UI. */
export const TASK_COLOR_CLASSES: Array<{
  bg: string;
  bgSoft: string;
  text: string;
  border: string;
  dot: string;
}> = [
  {
    bg: 'bg-sky-200',
    bgSoft: 'bg-sky-50',
    text: 'text-sky-950',
    border: 'border-sky-500',
    dot: 'bg-sky-500',
  },
  {
    bg: 'bg-amber-200',
    bgSoft: 'bg-amber-50',
    text: 'text-amber-950',
    border: 'border-amber-500',
    dot: 'bg-amber-500',
  },
  {
    bg: 'bg-emerald-200',
    bgSoft: 'bg-emerald-50',
    text: 'text-emerald-950',
    border: 'border-emerald-500',
    dot: 'bg-emerald-500',
  },
  {
    bg: 'bg-rose-200',
    bgSoft: 'bg-rose-50',
    text: 'text-rose-950',
    border: 'border-rose-500',
    dot: 'bg-rose-500',
  },
  {
    bg: 'bg-violet-200',
    bgSoft: 'bg-violet-50',
    text: 'text-violet-950',
    border: 'border-violet-500',
    dot: 'bg-violet-500',
  },
  {
    bg: 'bg-cyan-200',
    bgSoft: 'bg-cyan-50',
    text: 'text-cyan-950',
    border: 'border-cyan-500',
    dot: 'bg-cyan-500',
  },
  {
    bg: 'bg-orange-200',
    bgSoft: 'bg-orange-50',
    text: 'text-orange-950',
    border: 'border-orange-500',
    dot: 'bg-orange-500',
  },
  {
    bg: 'bg-teal-200',
    bgSoft: 'bg-teal-50',
    text: 'text-teal-950',
    border: 'border-teal-500',
    dot: 'bg-teal-500',
  },
];

/**
 * Advance by one repeat interval in the task timezone (mirrors Rust `add_one_interval`).
 * Uses calendar days in the zone so wall clock is preserved across DST.
 */
export function addOneInterval(
  utcIsoOrDate: string | Date,
  interval: ScheduleInterval | string,
  timeZone: string,
): Date {
  const utc = typeof utcIsoOrDate === 'string' ? new Date(utcIsoOrDate) : utcIsoOrDate;
  const wall = formatInTimeZone(utc, timeZone, "yyyy-MM-dd'T'HH:mm:ss");
  const [datePart, timePart = '00:00:00'] = wall.split('T');
  const [y, m, d] = datePart.split('-').map(Number);
  const [hh, mm, ss] = timePart.split(':').map(Number);

  let nextY = y;
  let nextM = m;
  let nextD = d;

  if (interval === 'daily') {
    const noon = new Date(Date.UTC(y, m - 1, d, 12, 0, 0));
    noon.setUTCDate(noon.getUTCDate() + 1);
    nextY = noon.getUTCFullYear();
    nextM = noon.getUTCMonth() + 1;
    nextD = noon.getUTCDate();
  } else if (interval === 'weekly') {
    const noon = new Date(Date.UTC(y, m - 1, d, 12, 0, 0));
    noon.setUTCDate(noon.getUTCDate() + 7);
    nextY = noon.getUTCFullYear();
    nextM = noon.getUTCMonth() + 1;
    nextD = noon.getUTCDate();
  } else {
    // Monthly: same day-of-month, clamped to last day of next month
    if (m === 12) {
      nextM = 1;
      nextY = y + 1;
    } else {
      nextM = m + 1;
      nextY = y;
    }
    const lastDay = daysInMonth(nextY, nextM);
    nextD = Math.min(d, lastDay);
  }

  const nextWall = `${pad(nextY, 4)}-${pad(nextM, 2)}-${pad(nextD, 2)}T${pad(hh, 2)}:${pad(mm, 2)}:${pad(ss, 2)}`;
  // Snap past DST spring gaps so calendar expansion matches Rust scheduling.
  return wallToUtc(nextWall, timeZone, { snapGap: true });
}

function daysInMonth(year: number, month: number): number {
  return new Date(Date.UTC(year, month, 0)).getUTCDate();
}

function pad(n: number, width: number): string {
  return String(n).padStart(width, '0');
}

function closeOffsetMs(task: ScheduleTaskInput): number | null {
  if (!task.close_time) {
    return null;
  }
  const start = new Date(task.start_time).getTime();
  const close = new Date(task.close_time).getTime();
  if (Number.isNaN(start) || Number.isNaN(close)) {
    return null;
  }
  return close - start;
}

function pushEvent(
  events: ScheduleEvent[],
  task: ScheduleTaskInput,
  action: ScheduleAction,
  at: Date,
  rangeStart: Date,
  rangeEnd: Date,
) {
  if (at < rangeStart || at >= rangeEnd) {
    return;
  }
  if (task.id == null) {
    return;
  }
  events.push({
    taskId: task.id,
    taskName: task.name,
    action,
    at,
    atIso: at.toISOString(),
  });
}

/**
 * Expand active tasks into open/close events within [rangeStart, rangeEnd).
 * Repeating tasks are walked with the same interval rules as the Rust scheduler.
 */
export function expandScheduleEvents(
  tasks: ScheduleTaskInput[],
  rangeStart: Date,
  rangeEnd: Date,
): ScheduleEvent[] {
  const events: ScheduleEvent[] = [];

  for (const task of tasks) {
    if (task.status !== 'active' || task.id == null) {
      continue;
    }

    const tz = task.timezone || 'UTC';
    const offset = closeOffsetMs(task);
    const endDate = task.repeat_config?.end_date
      ? new Date(task.repeat_config.end_date)
      : null;
    const endAfter = task.repeat_config?.end_after ?? null;

    if (!task.repeat_config) {
      if (task.next_open_execution) {
        const openAt = new Date(task.next_open_execution);
        pushEvent(events, task, 'open', openAt, rangeStart, rangeEnd);
      }
      if (task.next_close_execution) {
        const closeAt = new Date(task.next_close_execution);
        pushEvent(events, task, 'close', closeAt, rangeStart, rangeEnd);
      }
      continue;
    }

    let openAt: Date | null = task.next_open_execution
      ? new Date(task.next_open_execution)
      : null;

    // Mid-session close: open already fired, close still pending before next open.
    if (task.next_close_execution) {
      const pendingClose = new Date(task.next_close_execution);
      if (
        !Number.isNaN(pendingClose.getTime()) &&
        (!openAt || Number.isNaN(openAt.getTime()) || pendingClose < openAt)
      ) {
        pushEvent(events, task, 'close', pendingClose, rangeStart, rangeEnd);
      }
    }

    if (!openAt || Number.isNaN(openAt.getTime())) {
      continue;
    }

    let walkOpen: Date = openAt;
    const maxOpens =
      endAfter != null ? Math.max(0, endAfter - task.execution_count) : Number.POSITIVE_INFINITY;
    let opensShown = 0;

    // Bound iterations for safety (daily for ~3 years)
    for (let i = 0; i < 1200 && opensShown < maxOpens; i++) {
      if (walkOpen >= rangeEnd) {
        break;
      }
      if (endDate && !Number.isNaN(endDate.getTime()) && walkOpen >= endDate) {
        break;
      }

      if (walkOpen >= rangeStart) {
        pushEvent(events, task, 'open', walkOpen, rangeStart, rangeEnd);
        if (offset != null) {
          pushEvent(
            events,
            task,
            'close',
            new Date(walkOpen.getTime() + offset),
            rangeStart,
            rangeEnd,
          );
        }
      } else if (offset != null) {
        // Open was before the window; close may still land inside it.
        pushEvent(
          events,
          task,
          'close',
          new Date(walkOpen.getTime() + offset),
          rangeStart,
          rangeEnd,
        );
      }

      opensShown += 1;
      if (opensShown >= maxOpens) {
        break;
      }

      walkOpen = addOneInterval(walkOpen, task.repeat_config.interval, tz);
      if (endDate && !Number.isNaN(endDate.getTime()) && walkOpen >= endDate) {
        break;
      }
    }
  }

  events.sort((a, b) => a.at.getTime() - b.at.getTime());
  return events;
}

/** Local calendar day bounds for a Date (operator local). */
export function startOfLocalDay(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate());
}

export function addLocalDays(d: Date, days: number): Date {
  const next = new Date(d);
  next.setDate(next.getDate() + days);
  return next;
}

/** Monday-start week containing `d` (local). */
export function startOfLocalWeek(d: Date): Date {
  const day = startOfLocalDay(d);
  const dow = day.getDay(); // 0 Sun .. 6 Sat
  const mondayOffset = dow === 0 ? -6 : 1 - dow;
  return addLocalDays(day, mondayOffset);
}

export function startOfLocalMonth(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), 1);
}

export function addLocalMonths(d: Date, months: number): Date {
  return new Date(d.getFullYear(), d.getMonth() + months, 1);
}
