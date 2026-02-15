import { useState, useEffect } from 'react';
import { Task, BrowserType, TaskAction, TaskStatus, RepeatInterval } from '../types/task';

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
  const [formData, setFormData] = useState({
    name: '',
    browser: BrowserType.Chrome,
    action: TaskAction.Open,
    url: '',
    browserProfile: '',
    scheduledTime: '',
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    repeatEnabled: false,
    repeatInterval: RepeatInterval.Daily,
    repeatEndAfter: '',
    repeatEndDate: '',
  });

  useEffect(() => {
    if (initialTask) {
      setFormData({
        name: initialTask.name,
        browser: initialTask.browser,
        action: initialTask.action,
        url: initialTask.url || '',
        browserProfile: initialTask.browser_profile || '',
        scheduledTime: initialTask.scheduled_time ? new Date(initialTask.scheduled_time).toISOString().slice(0, 16) : '',
        timezone: initialTask.timezone,
        repeatEnabled: !!initialTask.repeat_config,
        repeatInterval: initialTask.repeat_config?.interval || RepeatInterval.Daily,
        repeatEndAfter: initialTask.repeat_config?.end_after?.toString() || '',
        repeatEndDate: initialTask.repeat_config?.end_date ? new Date(initialTask.repeat_config.end_date).toISOString().slice(0, 16) : '',
      });
    }
  }, [initialTask]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);

    try {
      const task: Task = {
        id: initialTask?.id,
        name: formData.name,
        browser: formData.browser,
        action: formData.action,
        url: formData.url || null,
        browser_profile: formData.browserProfile || null,
        scheduled_time: new Date(formData.scheduledTime).toISOString(),
        timezone: formData.timezone,
        repeat_config: formData.repeatEnabled
          ? {
              interval: formData.repeatInterval,
              end_after: formData.repeatEndAfter ? parseInt(formData.repeatEndAfter) : null,
              end_date: formData.repeatEndDate ? new Date(formData.repeatEndDate).toISOString() : null,
            }
          : null,
        status: initialTask?.status || TaskStatus.Active,
        created_at: initialTask?.created_at || new Date().toISOString(),
        updated_at: new Date().toISOString(),
        last_executed: initialTask?.last_executed,
        next_execution: initialTask?.next_execution,
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

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Browser
            <InfoTooltip text="Select which browser to control. Make sure the selected browser is installed on your system." />
          </label>
          <select
            value={formData.browser}
            onChange={(e) => setFormData({ ...formData, browser: e.target.value as BrowserType })}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          >
            {Object.values(BrowserType).map((browser) => (
              <option key={browser} value={browser} className="capitalize">
                {browser.charAt(0).toUpperCase() + browser.slice(1)}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Action
            <InfoTooltip text="Choose 'Open' to launch the browser or 'Close' to terminate all instances of the browser." />
          </label>
          <select
            value={formData.action}
            onChange={(e) => setFormData({ ...formData, action: e.target.value as TaskAction })}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          >
            <option value={TaskAction.Open}>Open</option>
            <option value={TaskAction.Close}>Close</option>
          </select>
        </div>
      </div>

      {formData.action === TaskAction.Open && (
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
      )}

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

      <div>
        <label className="flex items-center text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Scheduled Time
          <InfoTooltip text="The exact date and time when this task should execute. The task will run in your local timezone." />
        </label>
        <input
          type="datetime-local"
          required
          value={formData.scheduledTime}
          onChange={(e) => setFormData({ ...formData, scheduledTime: e.target.value })}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
        />
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
