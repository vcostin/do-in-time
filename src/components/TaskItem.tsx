import { useEffect, useState } from 'react';
import { Task, TaskStatus, BROWSER_LABELS, TaskExecutionLogEntry } from '../types/task';
import { format } from 'date-fns';
import { formatUtcInZone } from '../utils/datetime';
import { getSystemTimeZone } from '../utils/timezone';

interface TaskItemProps {
  task: Task;
  onEdit: (task: Task) => void;
  onDelete: (id: number) => void;
  onPause: (id: number) => void;
  onResume: (id: number) => void;
  onLoadLog: (id: number) => Promise<TaskExecutionLogEntry[]>;
}

export function TaskItem({
  task,
  onEdit,
  onDelete,
  onPause,
  onResume,
  onLoadLog,
}: TaskItemProps) {
  const [showLog, setShowLog] = useState(false);
  const [logEntries, setLogEntries] = useState<TaskExecutionLogEntry[]>([]);
  const [logLoading, setLogLoading] = useState(false);
  const [logError, setLogError] = useState<string | null>(null);

  const statusColors = {
    [TaskStatus.Active]: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
    [TaskStatus.Completed]: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
    [TaskStatus.Failed]: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
    [TaskStatus.Disabled]: 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300',
  };

  const localZone = getSystemTimeZone();
  const scheduleZone = task.timezone || localZone;
  const showScheduleZone = scheduleZone !== localZone;

  const formatDate = (dateStr: string) => {
    try {
      return format(new Date(dateStr), 'PPp');
    } catch {
      return dateStr;
    }
  };

  const formatScheduleDate = (dateStr: string) => {
    try {
      return `${formatUtcInZone(dateStr, scheduleZone)} ${scheduleZone}`;
    } catch {
      return dateStr;
    }
  };

  const TimeLine = ({ label, value }: { label: string; value: string }) => (
    <div className="flex items-start gap-2">
      <span className="font-medium shrink-0">{label}</span>
      <span className="min-w-0">
        <span className="block">{formatDate(value)}</span>
        {showScheduleZone && (
          <span className="block text-xs text-gray-500 dark:text-gray-500">
            {formatScheduleDate(value)}
          </span>
        )}
      </span>
    </div>
  );

  useEffect(() => {
    if (!showLog || !task.id) {
      return;
    }

    let cancelled = false;
    setLogLoading(true);
    setLogError(null);

    onLoadLog(task.id)
      .then((entries) => {
        if (!cancelled) {
          setLogEntries(entries);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setLogError(err instanceof Error ? err.message : 'Failed to load log');
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLogLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [showLog, task.id, task.last_execution_at, onLoadLog]);

  const canPause = task.id != null && task.status === TaskStatus.Active;
  const canResume =
    task.id != null &&
    (task.status === TaskStatus.Disabled || task.status === TaskStatus.Failed);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 hover:shadow-lg transition-shadow">
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 mb-2 flex-wrap">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {task.name}
            </h3>
            <span
              className={`px-2 py-1 text-xs font-medium rounded-full capitalize ${statusColors[task.status]}`}
            >
              {task.status}
            </span>
          </div>

          <div className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
            <div className="flex items-center gap-2">
              <span className="font-medium">Browser:</span>
              <span>{BROWSER_LABELS[task.browser] ?? task.browser}</span>
            </div>

            {task.url && (
              <div className="flex items-center gap-2">
                <span className="font-medium">URL:</span>
                <span className="truncate max-w-md">{task.url}</span>
              </div>
            )}

            <TimeLine label="Start Time:" value={task.start_time} />

            {task.close_time && <TimeLine label="Close Time:" value={task.close_time} />}

            {task.next_open_execution && task.status === TaskStatus.Active && (
              <TimeLine label="Next Open:" value={task.next_open_execution} />
            )}

            {task.next_close_execution && task.status === TaskStatus.Active && (
              <TimeLine label="Next Close:" value={task.next_close_execution} />
            )}

            <div className="flex items-center gap-2">
              <span className="font-medium">Schedule TZ:</span>
              <span>{scheduleZone}</span>
            </div>

            {task.repeat_config && (
              <div className="flex items-center gap-2">
                <span className="font-medium">Repeat:</span>
                <span className="capitalize">{task.repeat_config.interval}</span>
              </div>
            )}

            {task.last_execution_at && (
              <div className="flex items-center gap-2">
                <span className="font-medium">Last run:</span>
                <span>{formatDate(task.last_execution_at)}</span>
              </div>
            )}
          </div>

          {task.last_error && (
            <div className="mt-3 rounded-md border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20 p-3">
              <p className="text-xs font-medium text-red-800 dark:text-red-200 mb-1">
                Last error
              </p>
              <p className="text-sm text-red-700 dark:text-red-300 break-words">
                {task.last_error}
              </p>
            </div>
          )}

          {showLog && (
            <div className="mt-3 rounded-md border border-gray-200 dark:border-gray-700 p-3">
              <p className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">
                Recent execution log
              </p>
              {logLoading ? (
                <p className="text-sm text-gray-500">Loading…</p>
              ) : logError ? (
                <p className="text-sm text-red-600 dark:text-red-400">{logError}</p>
              ) : logEntries.length === 0 ? (
                <p className="text-sm text-gray-500">No runs logged yet.</p>
              ) : (
                <ul className="space-y-2 max-h-48 overflow-y-auto">
                  {logEntries.map((entry) => (
                    <li
                      key={entry.id}
                      className="text-xs text-gray-600 dark:text-gray-400 border-b border-gray-100 dark:border-gray-700 pb-2 last:border-0"
                    >
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="font-medium capitalize">{entry.action}</span>
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
                      {entry.message && entry.message !== 'ok' && (
                        <p className="mt-1 break-words">{entry.message}</p>
                      )}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>

        <div className="flex flex-col gap-2 shrink-0">
          {canPause && (
            <button
              onClick={() => task.id && onPause(task.id)}
              className="px-3 py-1 text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
            >
              Pause
            </button>
          )}
          {canResume && (
            <button
              onClick={() => task.id && onResume(task.id)}
              className="px-3 py-1 text-sm bg-emerald-100 dark:bg-emerald-900 text-emerald-800 dark:text-emerald-200 rounded hover:bg-emerald-200 dark:hover:bg-emerald-800 transition-colors"
            >
              {task.status === TaskStatus.Failed ? 'Retry' : 'Resume'}
            </button>
          )}
          <button
            onClick={() => setShowLog((v) => !v)}
            className="px-3 py-1 text-sm bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-200 rounded hover:bg-slate-200 dark:hover:bg-slate-600 transition-colors"
          >
            {showLog ? 'Hide log' : 'Log'}
          </button>
          <button
            onClick={() => onEdit(task)}
            className="px-3 py-1 text-sm bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded hover:bg-blue-200 dark:hover:bg-blue-800 transition-colors"
          >
            Edit
          </button>
          <button
            onClick={() => task.id && onDelete(task.id)}
            className="px-3 py-1 text-sm bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300 rounded hover:bg-red-200 dark:hover:bg-red-800 transition-colors"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  );
}
