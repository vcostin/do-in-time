import { useEffect, useState } from 'react';
import { Task, TaskStatus, BROWSER_LABELS, TaskExecutionLogEntry } from '../types/task';
import { formatUtcForDisplay, formatUtcInZone } from '../utils/datetime';
import { getSystemTimeZone } from '../utils/timezone';
import { useSettings } from '../hooks/useSettings';

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
  const { settings } = useSettings();
  const use24Hour = settings.use_24_hour_clock;
  const [showLog, setShowLog] = useState(false);
  const [showDetails, setShowDetails] = useState(false);
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
  const isActive = task.status === TaskStatus.Active;
  const showNextOpen = Boolean(task.next_open_execution && isActive);
  const showNextClose = Boolean(task.next_close_execution && isActive);

  const formatDate = (dateStr: string) => {
    try {
      return formatUtcForDisplay(dateStr, use24Hour);
    } catch {
      return dateStr;
    }
  };

  const formatScheduleDate = (dateStr: string) => {
    try {
      return `${formatUtcInZone(dateStr, scheduleZone, use24Hour)} ${scheduleZone}`;
    } catch {
      return dateStr;
    }
  };

  const TimeCell = ({ label, value }: { label: string; value: string }) => (
    <div className="min-w-0">
      <span className="text-xs font-medium text-gray-500 dark:text-gray-500">{label}</span>
      <span className="block text-sm text-gray-700 dark:text-gray-300 truncate" title={formatDate(value)}>
        {formatDate(value)}
      </span>
      {showScheduleZone && (
        <span
          className="block text-xs text-gray-500 dark:text-gray-500 truncate"
          title={formatScheduleDate(value)}
        >
          {formatScheduleDate(value)}
        </span>
      )}
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
  const hasDetails =
    showNextOpen ||
    showNextClose ||
    showScheduleZone ||
    Boolean(task.last_execution_at);

  const metaParts = [
    BROWSER_LABELS[task.browser] ?? task.browser,
    task.repeat_config
      ? task.repeat_config.interval.charAt(0).toUpperCase() +
        task.repeat_config.interval.slice(1)
      : null,
  ].filter(Boolean);

  const actionBtn =
    'px-2 py-0.5 text-xs rounded transition-colors';

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm px-3 py-2.5 hover:shadow transition-shadow">
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <h3 className="text-sm font-semibold text-gray-900 dark:text-white truncate">
              {task.name}
            </h3>
            <span
              className={`px-1.5 py-0.5 text-[11px] font-medium rounded capitalize ${statusColors[task.status]}`}
            >
              {task.status}
            </span>
          </div>

          <div className="mt-0.5 flex items-center gap-1.5 text-xs text-gray-500 dark:text-gray-400 min-w-0">
            <span className="shrink-0">{metaParts.join(' · ')}</span>
            {task.url && (
              <>
                <span className="shrink-0 text-gray-300 dark:text-gray-600">·</span>
                <span className="truncate" title={task.url}>
                  {task.url}
                </span>
              </>
            )}
          </div>

          <div className="mt-2 grid grid-cols-2 gap-x-4 gap-y-1.5">
            {showNextOpen && task.next_open_execution ? (
              <TimeCell label="Next open" value={task.next_open_execution} />
            ) : (
              <TimeCell label="Start" value={task.start_time} />
            )}
            {showNextClose && task.next_close_execution ? (
              <TimeCell label="Next close" value={task.next_close_execution} />
            ) : task.close_time ? (
              <TimeCell label="Close" value={task.close_time} />
            ) : (
              <div />
            )}
          </div>

          {(showNextOpen || showNextClose) && showDetails && (
            <div className="mt-1.5 grid grid-cols-2 gap-x-4 gap-y-1.5">
              <TimeCell label="Start" value={task.start_time} />
              {task.close_time ? (
                <TimeCell label="Close" value={task.close_time} />
              ) : (
                <div />
              )}
            </div>
          )}

          {showDetails && (showScheduleZone || task.last_execution_at) && (
            <div className="mt-1.5 flex flex-wrap gap-x-3 gap-y-0.5 text-xs text-gray-500 dark:text-gray-500">
              {showScheduleZone && <span>Schedule TZ: {scheduleZone}</span>}
              {task.last_execution_at && (
                <span>Last run: {formatDate(task.last_execution_at)}</span>
              )}
            </div>
          )}

          {task.last_error && (
            <div className="mt-2 rounded border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20 px-2 py-1.5">
              <p className="text-[11px] font-medium text-red-800 dark:text-red-200">
                Last error
              </p>
              <p className="text-xs text-red-700 dark:text-red-300 break-words line-clamp-2">
                {task.last_error}
              </p>
            </div>
          )}

          {showLog && (
            <div className="mt-2 rounded border border-gray-200 dark:border-gray-700 px-2 py-1.5">
              <p className="text-[11px] font-medium text-gray-700 dark:text-gray-300 mb-1">
                Recent execution log
              </p>
              {logLoading ? (
                <p className="text-xs text-gray-500">Loading…</p>
              ) : logError ? (
                <p className="text-xs text-red-600 dark:text-red-400">{logError}</p>
              ) : logEntries.length === 0 ? (
                <p className="text-xs text-gray-500">No runs logged yet.</p>
              ) : (
                <ul className="space-y-1.5 max-h-36 overflow-y-auto">
                  {logEntries.map((entry) => (
                    <li
                      key={entry.id}
                      className="text-xs text-gray-600 dark:text-gray-400 border-b border-gray-100 dark:border-gray-700 pb-1.5 last:border-0"
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
                        <p className="mt-0.5 break-words">{entry.message}</p>
                      )}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>

        <div className="flex flex-wrap justify-end gap-1 shrink-0 max-w-[11rem]">
          {canPause && (
            <button
              onClick={() => task.id && onPause(task.id)}
              className={`${actionBtn} bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600`}
            >
              Pause
            </button>
          )}
          {canResume && (
            <button
              onClick={() => task.id && onResume(task.id)}
              className={`${actionBtn} bg-emerald-100 dark:bg-emerald-900 text-emerald-800 dark:text-emerald-200 hover:bg-emerald-200 dark:hover:bg-emerald-800`}
            >
              {task.status === TaskStatus.Failed ? 'Retry' : 'Resume'}
            </button>
          )}
          {hasDetails && (
            <button
              onClick={() => setShowDetails((v) => !v)}
              className={`${actionBtn} bg-gray-50 dark:bg-gray-800 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-600`}
              title="Show start/close, last run, schedule timezone"
            >
              {showDetails ? 'Less' : 'More'}
            </button>
          )}
          <button
            onClick={() => setShowLog((v) => !v)}
            className={`${actionBtn} bg-slate-100 dark:bg-slate-700 text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-slate-600`}
          >
            {showLog ? 'Hide log' : 'Log'}
          </button>
          <button
            onClick={() => onEdit(task)}
            className={`${actionBtn} bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 hover:bg-blue-200 dark:hover:bg-blue-800`}
          >
            Edit
          </button>
          <button
            onClick={() => task.id && onDelete(task.id)}
            className={`${actionBtn} bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300 hover:bg-red-200 dark:hover:bg-red-800`}
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  );
}
