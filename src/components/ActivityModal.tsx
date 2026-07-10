import { useCallback, useEffect, useState } from 'react';
import { format } from 'date-fns';
import { RecentExecutionLogEntry } from '../types/task';
import { TauriTaskService } from '../services/tauri-api';
import { listen } from '@tauri-apps/api/event';

interface ActivityModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ActivityModal({ isOpen, onClose }: ActivityModalProps) {
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
    if (!isOpen) {
      return;
    }
    load();
  }, [isOpen, load]);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    const unlisten = listen('task-updated', () => {
      load();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [isOpen, load]);

  if (!isOpen) {
    return null;
  }

  const formatDate = (dateStr: string) => {
    try {
      return format(new Date(dateStr), 'PPp');
    } catch {
      return dateStr;
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />
      <div className="relative w-full max-w-2xl max-h-[85vh] flex flex-col bg-white dark:bg-gray-800 rounded-lg shadow-xl">
        <div className="flex items-center justify-between gap-4 p-4 border-b border-gray-200 dark:border-gray-700">
          <div>
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
              Activity
            </h2>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
              Recent open/close runs across all tasks
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-2 text-gray-500 hover:text-gray-800 dark:hover:text-gray-200"
            aria-label="Close"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

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

        <div className="flex-1 overflow-y-auto p-4">
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
