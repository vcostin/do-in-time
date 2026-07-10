import { useMemo, useState } from 'react';
import { Task } from '../types/task';
import { formatUtcForDisplay } from '../utils/datetime';
import { useSettings } from '../hooks/useSettings';
import {
  TASK_COLOR_CLASSES,
  addLocalDays,
  addLocalMonths,
  expandScheduleEvents,
  startOfLocalDay,
  startOfLocalMonth,
  startOfLocalWeek,
  taskColorIndex,
  type ScheduleEvent,
} from '../utils/scheduleEvents';

type CalendarMode = 'day' | 'week' | 'month';

interface ScheduleCalendarProps {
  tasks: Task[];
  onEditTask: (task: Task) => void;
}

function shortTime(date: Date, use24Hour: boolean): string {
  try {
    return new Intl.DateTimeFormat(undefined, {
      hour: 'numeric',
      minute: '2-digit',
      hour12: !use24Hour,
    }).format(date);
  } catch {
    return formatUtcForDisplay(date.toISOString(), use24Hour);
  }
}

function dayKey(d: Date): string {
  return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
}

function eventsForDay(events: ScheduleEvent[], day: Date): ScheduleEvent[] {
  const start = startOfLocalDay(day);
  const end = addLocalDays(start, 1);
  return events.filter((e) => e.at >= start && e.at < end);
}

function rangeLabel(anchor: Date, mode: CalendarMode): string {
  if (mode === 'day') {
    return anchor.toLocaleDateString(undefined, {
      weekday: 'long',
      month: 'long',
      day: 'numeric',
      year: 'numeric',
    });
  }
  if (mode === 'week') {
    const start = startOfLocalWeek(anchor);
    const end = addLocalDays(start, 6);
    const sameMonth = start.getMonth() === end.getMonth();
    const startStr = start.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
    });
    const endStr = end.toLocaleDateString(undefined, {
      month: sameMonth ? undefined : 'short',
      day: 'numeric',
      year: 'numeric',
    });
    return `${startStr} – ${endStr}`;
  }
  return anchor.toLocaleDateString(undefined, { month: 'long', year: 'numeric' });
}

function EventChip({
  event,
  use24Hour,
  compact,
  onClick,
}: {
  event: ScheduleEvent;
  use24Hour: boolean;
  compact?: boolean;
  onClick: () => void;
}) {
  const colors = TASK_COLOR_CLASSES[taskColorIndex(event.taskId)];
  const isOpen = event.action === 'open';
  const past = event.at.getTime() < Date.now();

  return (
    <button
      type="button"
      onClick={onClick}
      title={`${event.taskName} · ${event.action} · ${formatUtcForDisplay(event.atIso, use24Hour)}`}
      className={`w-full text-left rounded border-l-4 px-1.5 py-0.5 transition-opacity hover:opacity-90 ${
        colors.border
      } ${isOpen ? colors.bg : `${colors.bgSoft} border border-dashed ${colors.border}`} ${
        colors.text
      } ${past ? 'opacity-55' : ''} ${compact ? 'text-[10px] leading-tight' : 'text-xs'}`}
    >
      <span className="font-medium tabular-nums">{shortTime(event.at, use24Hour)}</span>
      <span className="mx-1 opacity-70">{isOpen ? 'Open' : 'Close'}</span>
      <span className={compact ? 'block truncate' : 'truncate'}>{event.taskName}</span>
    </button>
  );
}

export function ScheduleCalendar({ tasks, onEditTask }: ScheduleCalendarProps) {
  const { settings } = useSettings();
  const use24Hour = settings.use_24_hour_clock;
  const [mode, setMode] = useState<CalendarMode>('week');
  const [anchor, setAnchor] = useState(() => startOfLocalDay(new Date()));

  const taskById = useMemo(() => {
    const map = new Map<number, Task>();
    for (const t of tasks) {
      if (t.id != null) {
        map.set(t.id, t);
      }
    }
    return map;
  }, [tasks]);

  const { rangeStart, rangeEnd } = useMemo(() => {
    if (mode === 'day') {
      const start = startOfLocalDay(anchor);
      return { rangeStart: start, rangeEnd: addLocalDays(start, 1) };
    }
    if (mode === 'week') {
      const start = startOfLocalWeek(anchor);
      return { rangeStart: start, rangeEnd: addLocalDays(start, 7) };
    }
    const start = startOfLocalMonth(anchor);
    // Include leading/trailing days shown in the month grid
    const gridStart = startOfLocalWeek(start);
    const monthEnd = addLocalMonths(start, 1);
    const gridEnd = addLocalDays(startOfLocalWeek(monthEnd), 7);
    return { rangeStart: gridStart, rangeEnd: gridEnd };
  }, [anchor, mode]);

  const events = useMemo(
    () => expandScheduleEvents(tasks, rangeStart, rangeEnd),
    [tasks, rangeStart, rangeEnd],
  );

  const today = startOfLocalDay(new Date());

  const goPrev = () => {
    if (mode === 'day') {
      setAnchor((a) => addLocalDays(a, -1));
    } else if (mode === 'week') {
      setAnchor((a) => addLocalDays(a, -7));
    } else {
      setAnchor((a) => addLocalMonths(startOfLocalMonth(a), -1));
    }
  };

  const goNext = () => {
    if (mode === 'day') {
      setAnchor((a) => addLocalDays(a, 1));
    } else if (mode === 'week') {
      setAnchor((a) => addLocalDays(a, 7));
    } else {
      setAnchor((a) => addLocalMonths(startOfLocalMonth(a), 1));
    }
  };

  const handleEventClick = (event: ScheduleEvent) => {
    const task = taskById.get(event.taskId);
    if (task) {
      onEditTask(task);
    }
  };

  const modeBtn = (m: CalendarMode, label: string) => (
    <button
      type="button"
      onClick={() => setMode(m)}
      className={`px-2.5 py-1 text-xs rounded-md transition-colors ${
        mode === m
          ? 'bg-blue-600 text-white'
          : 'text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
      }`}
    >
      {label}
    </button>
  );

  const weekDays = useMemo(() => {
    const start = mode === 'week' ? startOfLocalWeek(anchor) : startOfLocalDay(anchor);
    if (mode === 'day') {
      return [start];
    }
    return Array.from({ length: 7 }, (_, i) => addLocalDays(startOfLocalWeek(anchor), i));
  }, [anchor, mode]);

  const monthCells = useMemo(() => {
    if (mode !== 'month') {
      return [];
    }
    const start = startOfLocalWeek(startOfLocalMonth(anchor));
    return Array.from({ length: 42 }, (_, i) => addLocalDays(start, i));
  }, [anchor, mode]);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
      <div className="flex flex-wrap items-center justify-between gap-3 px-3 py-2.5 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={goPrev}
            className="px-2 py-1 text-sm rounded border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
            aria-label="Previous"
          >
            ‹
          </button>
          <button
            type="button"
            onClick={() => setAnchor(startOfLocalDay(new Date()))}
            className="px-2 py-1 text-xs rounded border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            Today
          </button>
          <button
            type="button"
            onClick={goNext}
            className="px-2 py-1 text-sm rounded border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700"
            aria-label="Next"
          >
            ›
          </button>
          <h2 className="ml-2 text-sm font-semibold text-gray-900 dark:text-white">
            {rangeLabel(anchor, mode)}
          </h2>
        </div>
        <div className="flex items-center gap-1 p-0.5 rounded-lg bg-gray-100 dark:bg-gray-900">
          {modeBtn('day', 'Day')}
          {modeBtn('week', 'Week')}
          {modeBtn('month', 'Month')}
        </div>
      </div>

      <div className="px-3 py-2 flex flex-wrap items-center gap-3 text-[11px] text-gray-500 dark:text-gray-400 border-b border-gray-100 dark:border-gray-700/80">
        <span className="inline-flex items-center gap-1">
          <span className="inline-block w-3 h-3 rounded-sm bg-sky-200 border-l-4 border-sky-500" />
          Open
        </span>
        <span className="inline-flex items-center gap-1">
          <span className="inline-block w-3 h-3 rounded-sm bg-sky-50 border border-dashed border-sky-500" />
          Close
        </span>
        <span>Colors = task · click an event to edit</span>
        {events.length === 0 && (
          <span className="text-gray-400">No active opens/closes in this range</span>
        )}
      </div>

      {mode === 'month' ? (
        <div className="p-2">
          <div className="grid grid-cols-7 gap-px mb-1">
            {['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'].map((d) => (
              <div
                key={d}
                className="text-center text-[11px] font-medium text-gray-500 dark:text-gray-400 py-1"
              >
                {d}
              </div>
            ))}
          </div>
          <div className="grid grid-cols-7 gap-1">
            {monthCells.map((day) => {
              const dayEvents = eventsForDay(events, day);
              const inMonth = day.getMonth() === anchor.getMonth();
              const isToday = dayKey(day) === dayKey(today);
              const uniqueTasks = new Set(dayEvents.map((e) => e.taskId));
              return (
                <button
                  type="button"
                  key={dayKey(day)}
                  onClick={() => {
                    setAnchor(startOfLocalDay(day));
                    setMode('day');
                  }}
                  className={`min-h-[4.5rem] rounded-md border p-1 text-left transition-colors hover:border-blue-400 dark:hover:border-blue-500 ${
                    inMonth
                      ? 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700'
                      : 'bg-gray-50 dark:bg-gray-900/50 border-transparent text-gray-400'
                  } ${isToday ? 'ring-2 ring-blue-500 ring-offset-1 dark:ring-offset-gray-800' : ''}`}
                >
                  <div
                    className={`text-xs font-medium mb-1 ${
                      isToday ? 'text-blue-600 dark:text-blue-400' : 'text-gray-700 dark:text-gray-300'
                    }`}
                  >
                    {day.getDate()}
                  </div>
                  <div className="flex flex-wrap gap-0.5">
                    {[...uniqueTasks].slice(0, 4).map((id) => {
                      const colors = TASK_COLOR_CLASSES[taskColorIndex(id)];
                      return (
                        <span
                          key={id}
                          className={`w-1.5 h-1.5 rounded-full ${colors.dot}`}
                          title={taskById.get(id)?.name}
                        />
                      );
                    })}
                    {uniqueTasks.size > 4 && (
                      <span className="text-[9px] text-gray-400">+{uniqueTasks.size - 4}</span>
                    )}
                  </div>
                  {dayEvents.length > 0 && (
                    <div className="mt-0.5 text-[9px] text-gray-500 dark:text-gray-400">
                      {dayEvents.length} event{dayEvents.length === 1 ? '' : 's'}
                    </div>
                  )}
                </button>
              );
            })}
          </div>
        </div>
      ) : (
        <div
          className={`grid gap-px bg-gray-200 dark:bg-gray-700 ${
            mode === 'week' ? 'grid-cols-7' : 'grid-cols-1'
          }`}
        >
          {weekDays.map((day) => {
            const dayEvents = eventsForDay(events, day);
            const isToday = dayKey(day) === dayKey(today);
            return (
              <div
                key={dayKey(day)}
                className="bg-white dark:bg-gray-800 min-h-[12rem] flex flex-col"
              >
                <div
                  className={`px-2 py-1.5 border-b border-gray-100 dark:border-gray-700 ${
                    isToday ? 'bg-blue-50 dark:bg-blue-950/40' : ''
                  }`}
                >
                  <div className="text-[11px] font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                    {day.toLocaleDateString(undefined, { weekday: 'short' })}
                  </div>
                  <div
                    className={`text-sm font-semibold ${
                      isToday
                        ? 'text-blue-700 dark:text-blue-300'
                        : 'text-gray-900 dark:text-white'
                    }`}
                  >
                    {day.getDate()}
                  </div>
                </div>
                <div className="flex-1 p-1.5 space-y-1 overflow-y-auto max-h-[28rem]">
                  {dayEvents.length === 0 ? (
                    <p className="text-[11px] text-gray-400 dark:text-gray-500 px-0.5 py-2">
                      —
                    </p>
                  ) : (
                    dayEvents.map((event) => (
                      <EventChip
                        key={`${event.taskId}-${event.action}-${event.atIso}`}
                        event={event}
                        use24Hour={use24Hour}
                        compact={mode === 'week'}
                        onClick={() => handleEventClick(event)}
                      />
                    ))
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
