import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Task, BrowserType, BROWSER_LABELS, TaskStatus, RepeatInterval } from '../types/task';
import {
  utcToZonedDatetimeString,
  zonedDatetimeStringToUtc,
} from '../utils/datetime';
import {
  extractTimezoneAbbr,
  getSystemTimeZone,
  resolveScheduleTimezone,
  scheduleTimezoneOptions,
} from '../utils/timezone';
import * as chrono from 'chrono-node';
import type { ParsedComponents } from 'chrono-node';
import { DateTimeLocalFields } from './DateTimeLocalFields';

interface TaskFormProps {
  initialTask: Task | null;
  onSubmit: (task: Task) => Promise<void>;
  onCancel: () => void;
}

const InfoTooltip = ({ text }: { text: string }) => (
  <div className="group relative inline-block ml-2">
    <svg
      className="w-4 h-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 cursor-help"
      fill="currentColor"
      viewBox="0 0 20 20"
    >
      <path
        fillRule="evenodd"
        d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
        clipRule="evenodd"
      />
    </svg>
    <div className="invisible group-hover:visible absolute left-6 top-0 z-10 w-64 p-2 bg-gray-900 text-white text-xs rounded-lg shadow-lg">
      {text}
      <div className="absolute left-0 top-2 -ml-1 w-2 h-2 bg-gray-900 transform rotate-45"></div>
    </div>
  </div>
);

function orderBrowsers(
  installed: BrowserType[],
  defaultBr: BrowserType | null,
  preferred?: BrowserType | null,
): BrowserType[] {
  const ordered = [...installed];

  const moveToFront = (browser: BrowserType | null | undefined) => {
    if (!browser) return;
    const index = ordered.indexOf(browser);
    if (index > 0) {
      ordered.splice(index, 1);
      ordered.unshift(browser);
    } else if (index === -1) {
      ordered.unshift(browser);
    }
  };

  // Keep edit selection available even if detection missed it, then prefer system default.
  moveToFront(preferred);
  moveToFront(defaultBr);
  return ordered;
}

function wallClockFromChrono(component: ParsedComponents): string {
  const year = component.get('year');
  const month = String(component.get('month')).padStart(2, '0');
  const day = String(component.get('day')).padStart(2, '0');
  const hour = String(component.get('hour') ?? 0).padStart(2, '0');
  const minute = String(component.get('minute') ?? 0).padStart(2, '0');
  return `${year}-${month}-${day}T${hour}:${minute}`;
}

export function TaskForm({ initialTask, onSubmit, onCancel }: TaskFormProps) {
  const [submitting, setSubmitting] = useState(false);
  const [detectingBrowsers, setDetectingBrowsers] = useState(true);
  const [browserError, setBrowserError] = useState<string | null>(null);
  const [installedBrowsers, setInstalledBrowsers] = useState<BrowserType[]>([]);
  const [defaultBrowser, setDefaultBrowser] = useState<BrowserType | null>(null);
  const [naturalLanguageTime, setNaturalLanguageTime] = useState('');
  const [nlError, setNlError] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    browser: '' as BrowserType | '',
    url: '',
    allowCloseAll: false,
    startTime: '',
    closeTime: '',
    timezone: getSystemTimeZone(),
    repeatEnabled: false,
    repeatInterval: RepeatInterval.Daily,
    repeatEndAfter: '',
    repeatEndDate: '',
  });

  const timezoneOptions = useMemo(
    () => scheduleTimezoneOptions(formData.timezone),
    [formData.timezone],
  );

  // Detect installed browsers on mount
  useEffect(() => {
    const detectBrowsers = async () => {
      setDetectingBrowsers(true);
      setBrowserError(null);

      try {
        const [installed, defaultBr] = await Promise.all([
          invoke<BrowserType[]>('get_installed_browsers'),
          invoke<BrowserType | null>('get_default_browser'),
        ]);

        const ordered = orderBrowsers(installed, defaultBr, initialTask?.browser);
        setInstalledBrowsers(ordered);
        setDefaultBrowser(defaultBr);

        if (!initialTask) {
          const preferred =
            (defaultBr && ordered.includes(defaultBr) && defaultBr) ||
            ordered[0] ||
            '';
          setFormData(prev => ({ ...prev, browser: preferred }));
        }

        if (ordered.length === 0) {
          setBrowserError('No supported browsers were detected on this system.');
        }
      } catch (error) {
        console.error('Failed to detect browsers:', error);
        setInstalledBrowsers([]);
        setBrowserError('Could not detect installed browsers. Check app permissions and try again.');
      } finally {
        setDetectingBrowsers(false);
      }
    };

    detectBrowsers();
  }, [initialTask]);

  useEffect(() => {
    if (initialTask) {
      const tz = initialTask.timezone || getSystemTimeZone();
      setFormData({
        name: initialTask.name,
        browser: initialTask.browser,
        url: initialTask.url || '',
        allowCloseAll: initialTask.allow_close_all || false,
        startTime: initialTask.start_time
          ? utcToZonedDatetimeString(initialTask.start_time, tz)
          : '',
        closeTime: initialTask.close_time
          ? utcToZonedDatetimeString(initialTask.close_time, tz)
          : '',
        timezone: tz,
        repeatEnabled: !!initialTask.repeat_config,
        repeatInterval: initialTask.repeat_config?.interval || RepeatInterval.Daily,
        repeatEndAfter: initialTask.repeat_config?.end_after?.toString() || '',
        repeatEndDate: initialTask.repeat_config?.end_date
          ? utcToZonedDatetimeString(initialTask.repeat_config.end_date, tz)
          : '',
      });
    }
  }, [initialTask]);

  const handleNaturalLanguageInput = (input: string) => {
    setNaturalLanguageTime(input);
    setNlError(null);

    if (!input.trim()) {
      return;
    }

    const results = chrono.parse(input);
    if (results.length === 0 || !results[0].start) {
      setNlError('Could not understand that time. Try e.g. "tomorrow at 2pm PT" or "Jan 31 from 9am to 11am ET".');
      return;
    }

    const parsed = results[0];
    const abbr = extractTimezoneAbbr(parsed.text) ?? extractTimezoneAbbr(input);
    const hasExplicitZone = parsed.start.isCertain('timezoneOffset');

    if (hasExplicitZone && abbr) {
      const resolved = resolveScheduleTimezone({
        abbr,
        fallbackIana: formData.timezone,
      });
      if (resolved.error) {
        setNlError(resolved.error);
        return;
      }

      const scheduleTz = resolved.iana;
      const startUtc = parsed.start.date().toISOString();
      const updates: Partial<typeof formData> = {
        timezone: scheduleTz,
        startTime: utcToZonedDatetimeString(startUtc, scheduleTz),
      };
      if (parsed.end) {
        updates.closeTime = utcToZonedDatetimeString(parsed.end.date().toISOString(), scheduleTz);
      }
      setFormData((prev) => ({ ...prev, ...updates }));
      return;
    }

    if (hasExplicitZone && !abbr) {
      setNlError(
        'Timezone abbreviation was not recognized. Pick a schedule timezone below, or use a known abbr like ET, PT, CET, JST.',
      );
      return;
    }

    // No zone in phrase: treat chrono wall-clock components as schedule-zone local time.
    const scheduleTz = formData.timezone;
    const startWall = wallClockFromChrono(parsed.start);
    const startUtc = zonedDatetimeStringToUtc(startWall, scheduleTz);
    const updates: Partial<typeof formData> = {
      startTime: utcToZonedDatetimeString(startUtc, scheduleTz),
    };
    if (parsed.end) {
      const endWall = wallClockFromChrono(parsed.end);
      const endUtc = zonedDatetimeStringToUtc(endWall, scheduleTz);
      updates.closeTime = utcToZonedDatetimeString(endUtc, scheduleTz);
    }
    setFormData((prev) => ({ ...prev, ...updates }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.browser) {
      setBrowserError('Select an installed browser before saving.');
      return;
    }
    setSubmitting(true);

    try {
      const tz = formData.timezone;
      const task: Task = {
        id: initialTask?.id,
        name: formData.name,
        browser: formData.browser,
        url: formData.url || null,
        allow_close_all: formData.allowCloseAll,
        // Profile picker is deferred (BACKLOG); preserve any existing value on edit.
        browser_profile: initialTask?.browser_profile ?? null,
        start_time: zonedDatetimeStringToUtc(formData.startTime, tz),
        close_time: formData.closeTime ? zonedDatetimeStringToUtc(formData.closeTime, tz) : null,
        timezone: tz,
        repeat_config: formData.repeatEnabled
          ? {
              interval: formData.repeatInterval,
              end_after: formData.repeatEndAfter ? parseInt(formData.repeatEndAfter) : null,
              end_date: formData.repeatEndDate
                ? zonedDatetimeStringToUtc(formData.repeatEndDate, tz)
                : null,
            }
          : null,
        execution_count: initialTask?.execution_count || 0,
        status: initialTask?.status || TaskStatus.Active,
        next_open_execution: initialTask?.next_open_execution,
        next_close_execution: initialTask?.next_close_execution,
        last_error: initialTask?.last_error ?? null,
        last_execution_at: initialTask?.last_execution_at ?? null,
      };

      await onSubmit(task);
    } catch (err) {
      console.error('Failed to submit task:', err);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          {initialTask ? 'Edit Task' : 'Create New Task'}
        </h2>
      </div>

      <div>
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Task Name
          <InfoTooltip text="A descriptive name for your task to help you identify it later. Example: 'Open Chrome for work' or 'Close Firefox at end of day'." />
        </label>
        <input
          type="text"
          required
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          placeholder="e.g., Open Chrome for work"
        />
      </div>

      <div className="grid grid-cols-1 gap-4">
        <div>
          <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Browser
            <InfoTooltip text="Only browsers detected on this system are listed. The system default browser is selected automatically for new tasks." />
          </label>
          <select
            required
            disabled={detectingBrowsers || installedBrowsers.length === 0}
            value={formData.browser}
            onChange={(e) => setFormData({ ...formData, browser: e.target.value as BrowserType })}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 disabled:opacity-60"
          >
            {detectingBrowsers && <option value="">Detecting installed browsers...</option>}
            {!detectingBrowsers && installedBrowsers.length === 0 && (
              <option value="">No installed browsers found</option>
            )}
            {installedBrowsers.map((browser) => (
              <option key={browser} value={browser}>
                {BROWSER_LABELS[browser]}
                {browser === defaultBrowser ? ' (Default)' : ''}
              </option>
            ))}
          </select>
          {detectingBrowsers && (
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Detecting installed browsers...
            </p>
          )}
          {browserError && (
            <p className="mt-1 text-xs text-red-600 dark:text-red-400">{browserError}</p>
          )}
        </div>
      </div>

      <div>
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          URL (optional)
          <InfoTooltip text="The website to open when launching the browser. Leave empty to open the browser's default home page." />
        </label>
        <input
          type="url"
          value={formData.url}
          onChange={(e) => setFormData({ ...formData, url: e.target.value })}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          placeholder="https://example.com"
        />
      </div>

      <div>
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Schedule timezone
          <InfoTooltip text="Wall-clock times below are in this timezone. Stored as UTC in the database. Repeating tasks keep the same local time across DST in this zone. Natural-language phrases like ET or JST update this field." />
        </label>
        <select
          value={formData.timezone}
          onChange={(e) => setFormData({ ...formData, timezone: e.target.value })}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
        >
          {timezoneOptions.map((tz) => (
            <option key={tz} value={tz}>
              {tz === getSystemTimeZone() ? `${tz} (system)` : tz}
            </option>
          ))}
        </select>
        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Changing the zone keeps the typed clock times; the UTC instant updates on save.
        </p>
      </div>

      <div className="border-2 border-blue-200 dark:border-blue-800 rounded-lg p-4 bg-blue-50 dark:bg-blue-900/20">
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Quick Time Entry (optional)
          <InfoTooltip text="Natural language like 'January 31st from 9am to 11am ET', 'tomorrow at 2pm PT', or 'March 15 9:00 JST'. Zone abbreviations set the schedule timezone; without a zone, times use the schedule timezone selected above." />
        </label>
        <input
          type="text"
          value={naturalLanguageTime}
          onChange={(e) => handleNaturalLanguageInput(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          placeholder="e.g., January 31st from 9am to 11am ET"
        />
        {nlError ? (
          <p className="mt-1 text-xs text-red-600 dark:text-red-400">{nlError}</p>
        ) : (
          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
            Try: "next Friday at 3pm", "tomorrow from 9am to 5pm PT", "Jan 15th at 2:30pm JST"
          </p>
        )}
      </div>

      <div>
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Start Time
          <InfoTooltip text="Wall-clock open time in the schedule timezone above. Saved as UTC; repeats follow that zone's DST rules." />
        </label>
        <DateTimeLocalFields
          required
          value={formData.startTime}
          onChange={(startTime) => setFormData({ ...formData, startTime })}
        />
      </div>

      <div>
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Close Time (optional)
          <InfoTooltip text="Optional wall-clock close time in the schedule timezone. Leave empty if you don't want to automatically close the browser." />
        </label>
        <DateTimeLocalFields
          value={formData.closeTime}
          onChange={(closeTime) => setFormData({ ...formData, closeTime })}
        />
      </div>

      <div>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={formData.allowCloseAll}
            onChange={(e) => setFormData({ ...formData, allowCloseAll: e.target.checked })}
            className="w-4 h-4 text-red-600 border-gray-300 rounded focus:ring-red-500"
          />
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Allow close all browser instances (dangerous)
          </span>
          <InfoTooltip text="On Windows/Linux, URL close matches window titles containing the site host. If no window matches, enabling this falls back to terminating all instances of the selected browser. Also required to close when the task has no URL." />
        </label>
      </div>

      <div>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={formData.repeatEnabled}
            onChange={(e) => setFormData({ ...formData, repeatEnabled: e.target.checked })}
            className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
          />
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Repeat task
          </span>
          <InfoTooltip text="Enable this to make the task repeat automatically at regular intervals (daily, weekly, or monthly)." />
        </label>
      </div>

      {formData.repeatEnabled && (
        <div className="pl-6 space-y-4 border-l-2 border-blue-500">
          <div>
            <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Repeat Interval
              <InfoTooltip text="How often the task should repeat: Daily (every 24 hours), Weekly (every 7 days), or Monthly (same day each month)." />
            </label>
            <select
              value={formData.repeatInterval}
              onChange={(e) => setFormData({ ...formData, repeatInterval: e.target.value as RepeatInterval })}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            >
              <option value={RepeatInterval.Daily}>Daily</option>
              <option value={RepeatInterval.Weekly}>Weekly</option>
              <option value={RepeatInterval.Monthly}>Monthly</option>
            </select>
          </div>

          <div>
            <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              End after (occurrences, optional)
              <InfoTooltip text="Stop repeating after this many executions. For example, '10' means the task will run 10 times then stop. Leave empty for unlimited repetitions." />
            </label>
            <input
              type="number"
              min="1"
              value={formData.repeatEndAfter}
              onChange={(e) => setFormData({ ...formData, repeatEndAfter: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              placeholder="e.g., 10"
            />
          </div>

          <div>
            <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              End date (optional)
              <InfoTooltip text="Stop repeating after this date and time (in the schedule timezone). Leave empty for unlimited repetitions." />
            </label>
            <DateTimeLocalFields
              value={formData.repeatEndDate}
              onChange={(repeatEndDate) => setFormData({ ...formData, repeatEndDate })}
            />
          </div>
        </div>
      )}

      <div className="flex gap-3 pt-4">
        <button
          type="submit"
          disabled={submitting}
          className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {submitting ? 'Saving...' : initialTask ? 'Update Task' : 'Create Task'}
        </button>
        <button
          type="button"
          onClick={onCancel}
          disabled={submitting}
          className="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          Cancel
        </button>
      </div>
    </form>
  );
}
