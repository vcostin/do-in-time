import { useCallback, useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { RecentExecutionLogEntry } from '../types/task';
import { TauriTaskService } from '../services/tauri-api';
import { formatUtcForDisplay } from '../utils/datetime';
import { useSettings } from '../hooks/useSettings';
import { PageHeader } from '../components/PageHeader';

export function ActivityPage() {
  const { settings } = useSettings();
  const [entries, setEntries] = useState<RecentExecutionLogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [failuresOnly, setFailuresOnly] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await TauriTaskService.getRecentExecutionLog(50, failuresOnly);
      setEntries(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load activity');
    } finally {
      setLoading(false);
    }
  }, [failuresOnly]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    const unlisten = listen('task-updated', () => {
      load();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [load]);

  const formatDate = (dateStr: string) => {
    try {
      return formatUtcForDisplay(dateStr, settings.use_24_hour_clock);
    } catch {
      return dateStr;
    }
  };

  return (
    <div>
      <PageHeader
        title="Activity"
        description="Recent open/close runs across all tasks"
        backTo="/"
        backLabel="Back to list"
      />

      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between gap-3 px-4 py-3 border-b border-gray-100 dark:border-gray-700">
          <label className="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
            <input
              type="checkbox"
              checked={failuresOnly}
              onChange={(e) => setFailuresOnly(e.target.checked)}
              className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
            />
            Failures only
          </label>
          <button
            type="button"
            onClick={load}
            className="text-sm text-blue-600 dark:text-blue-400 hover:underline"
          >
            Refresh
          </button>
        </div>

        <div className="p-4 max-h-[70vh] overflow-y-auto">
          {loading ? (
            <p className="text-sm text-gray-500">Loading…</p>
          ) : error ? (
            <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
          ) : entries.length === 0 ? (
            <p className="text-sm text-gray-500">
              {failuresOnly ? 'No failed runs logged yet.' : 'No runs logged yet.'}
            </p>
          ) : (
            <ul className="space-y-3">
              {entries.map((entry) => (
                <li
                  key={entry.id}
                  className="rounded-md border border-gray-200 dark:border-gray-700 p-3"
                >
                  <div className="flex items-start justify-between gap-3 flex-wrap">
                    <div className="min-w-0">
                      <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                        {entry.task_name}
                      </p>
                      <div className="mt-1 flex items-center gap-2 flex-wrap text-xs text-gray-600 dark:text-gray-400">
                        <span className="capitalize font-medium">{entry.action}</span>
                        <span
                          className={
                            entry.success
                              ? 'text-green-700 dark:text-green-400'
                              : 'text-red-700 dark:text-red-400'
                          }
                        >
                          {entry.success ? 'ok' : 'failed'}
                        </span>
                        <span>{formatDate(entry.created_at)}</span>
                      </div>
                    </div>
                  </div>
                  {entry.message && entry.message !== 'ok' && (
                    <p className="mt-2 text-sm text-gray-700 dark:text-gray-300 break-words">
                      {entry.message}
                    </p>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}
