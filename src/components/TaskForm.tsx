import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Task, BrowserType, TaskStatus, RepeatInterval } from '../types/task';
import { utcToLocalDatetimeString, localDatetimeStringToUtc } from '../utils/datetime';
import * as chrono from 'chrono-node';

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

export function TaskForm({ initialTask, onSubmit, onCancel }: TaskFormProps) {
  const [submitting, setSubmitting] = useState(false);
  const [installedBrowsers, setInstalledBrowsers] = useState<BrowserType[]>([]);
  const [defaultBrowser, setDefaultBrowser] = useState<BrowserType | null>(null);
  const [naturalLanguageTime, setNaturalLanguageTime] = useState('');
  const [formData, setFormData] = useState({
    name: '',
    browser: BrowserType.Chrome,
    url: '',
    allowCloseAll: false,
    browserProfile: '',
    startTime: '',
    closeTime: '',
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    repeatEnabled: false,
    repeatInterval: RepeatInterval.Daily,
    repeatEndAfter: '',
    repeatEndDate: '',
  });

  // Detect installed browsers on mount
  useEffect(() => {
    const detectBrowsers = async () => {
      try {
        const installed = await invoke<BrowserType[]>('get_installed_browsers');
        const defaultBr = await invoke<BrowserType | null>('get_default_browser');

        setInstalledBrowsers(installed);
        setDefaultBrowser(defaultBr);

        // Set default browser if available and not editing an existing task
        if (!initialTask && defaultBr && installed.includes(defaultBr)) {
          setFormData(prev => ({ ...prev, browser: defaultBr }));
        } else if (!initialTask && installed.length > 0) {
          // If no default detected, use first installed browser
          setFormData(prev => ({ ...prev, browser: installed[0] }));
        }
      } catch (error) {
        console.error('Failed to detect browsers:', error);
        // Fallback: use all browsers if detection fails
        setInstalledBrowsers(Object.values(BrowserType));
      }
    };

    detectBrowsers();
  }, []);

  useEffect(() => {
    if (initialTask) {
      setFormData({
        name: initialTask.name,
        browser: initialTask.browser,
        url: initialTask.url || '',
        allowCloseAll: initialTask.allow_close_all || false,
        browserProfile: initialTask.browser_profile || '',
        startTime: initialTask.start_time ? utcToLocalDatetimeString(initialTask.start_time) : '',
        closeTime: initialTask.close_time ? utcToLocalDatetimeString(initialTask.close_time) : '',
        timezone: initialTask.timezone,
        repeatEnabled: !!initialTask.repeat_config,
        repeatInterval: initialTask.repeat_config?.interval || RepeatInterval.Daily,
        repeatEndAfter: initialTask.repeat_config?.end_after?.toString() || '',
        repeatEndDate: initialTask.repeat_config?.end_date ? utcToLocalDatetimeString(initialTask.repeat_config.end_date) : '',
      });
    }
  }, [initialTask]);

  const handleNaturalLanguageInput = (input: string) => {
    setNaturalLanguageTime(input);

    if (!input.trim()) return;

    // Parse the natural language date/time string
    const results = chrono.parse(input);

    if (results.length > 0) {
      const parsed = results[0];

      // Get start time
      if (parsed.start) {
        const startDate = parsed.start.date();
        const startTimeStr = utcToLocalDatetimeString(startDate.toISOString());
        setFormData(prev => ({ ...prev, startTime: startTimeStr }));
      }

      // Get end time if available (for "from X to Y" patterns)
      if (parsed.end) {
        const endDate = parsed.end.date();
        const endTimeStr = utcToLocalDatetimeString(endDate.toISOString());
        setFormData(prev => ({ ...prev, closeTime: endTimeStr }));
      }
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);

    try {
      const task: Task = {
        id: initialTask?.id,
        name: formData.name,
        browser: formData.browser,
        url: formData.url || null,
        allow_close_all: formData.allowCloseAll,
        browser_profile: formData.browserProfile || null,
        start_time: localDatetimeStringToUtc(formData.startTime),
        close_time: formData.closeTime ? localDatetimeStringToUtc(formData.closeTime) : null,
        timezone: formData.timezone,
        repeat_config: formData.repeatEnabled
          ? {
              interval: formData.repeatInterval,
              end_after: formData.repeatEndAfter ? parseInt(formData.repeatEndAfter) : null,
              end_date: formData.repeatEndDate ? localDatetimeStringToUtc(formData.repeatEndDate) : null,
            }
          : null,
        execution_count: initialTask?.execution_count || 0,
        status: initialTask?.status || TaskStatus.Active,
        next_open_execution: initialTask?.next_open_execution,
        next_close_execution: initialTask?.next_close_execution,
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
            <InfoTooltip text="Select which browser to control. Only browsers detected on your system are shown." />
          </label>
          <select
            value={formData.browser}
            onChange={(e) => setFormData({ ...formData, browser: e.target.value as BrowserType })}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          >
            {(installedBrowsers.length > 0 ? installedBrowsers : Object.values(BrowserType)).map((browser) => (
              <option key={browser} value={browser} className="capitalize">
                {browser.charAt(0).toUpperCase() + browser.slice(1)}
                {browser === defaultBrowser ? ' (Default)' : ''}
              </option>
            ))}
          </select>
          {installedBrowsers.length === 0 && (
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Detecting installed browsers...
            </p>
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
          Browser Profile (optional)
          <InfoTooltip text="For Chrome/Edge: use 'Default', 'Profile 1', 'Profile 2', etc. For Firefox: enter the profile name. Leave empty to use the default profile." />
        </label>
        <input
          type="text"
          value={formData.browserProfile}
          onChange={(e) => setFormData({ ...formData, browserProfile: e.target.value })}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          placeholder="e.g., Profile 1"
        />
      </div>

      <div className="border-2 border-blue-200 dark:border-blue-800 rounded-lg p-4 bg-blue-50 dark:bg-blue-900/20">
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Quick Time Entry (optional)
          <InfoTooltip text="Enter times in natural language like 'January 31st from 9am to 11am ET' or 'tomorrow at 2pm to 4pm'. This will automatically fill the Start and Close Time fields below." />
        </label>
        <input
          type="text"
          value={naturalLanguageTime}
          onChange={(e) => handleNaturalLanguageInput(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          placeholder="e.g., January 31st from 9am to 11am ET"
        />
        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Try: "next Friday at 3pm", "tomorrow from 9am to 5pm", "Jan 15th at 2:30pm"
        </p>
      </div>

      <div>
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Start Time
          <InfoTooltip text="The exact date and time when the browser should open. The task will run in your local timezone." />
        </label>
        <input
          type="datetime-local"
          required
          value={formData.startTime}
          onChange={(e) => setFormData({ ...formData, startTime: e.target.value })}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
        />
      </div>

      <div>
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Close Time (optional)
          <InfoTooltip text="Optional: date and time when the browser should automatically close. Leave empty if you don't want to automatically close the browser." />
        </label>
        <input
          type="datetime-local"
          value={formData.closeTime}
          onChange={(e) => setFormData({ ...formData, closeTime: e.target.value })}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
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
          <InfoTooltip text="When enabled, a Close action with no URL will terminate all instances of the selected browser. Leave this disabled unless you explicitly need that behavior." />
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
              <InfoTooltip text="Stop repeating after this date and time. The task will not execute beyond this point. Leave empty for unlimited repetitions." />
            </label>
            <input
              type="datetime-local"
              value={formData.repeatEndDate}
              onChange={(e) => setFormData({ ...formData, repeatEndDate: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
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
