import { Task, TaskAction, TaskStatus } from '../types/task';
import { format } from 'date-fns';

interface TaskItemProps {
  task: Task;
  onEdit: (task: Task) => void;
  onDelete: (id: number) => void;
}

export function TaskItem({ task, onEdit, onDelete }: TaskItemProps) {
  const statusColors = {
    [TaskStatus.Pending]: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300',
    [TaskStatus.Active]: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
    [TaskStatus.Completed]: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
    [TaskStatus.Failed]: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
    [TaskStatus.Disabled]: 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400',
  };

  const formatDate = (dateStr: string) => {
    try {
      return format(new Date(dateStr), 'PPp');
    } catch {
      return dateStr;
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 hover:shadow-lg transition-shadow">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-3 mb-2">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {task.name}
            </h3>
            <span className={`px-2 py-1 text-xs font-medium rounded-full ${statusColors[task.status]}`}>
              {task.status}
            </span>
          </div>

          <div className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
            <div className="flex items-center gap-2">
              <span className="font-medium">Browser:</span>
              <span className="capitalize">{task.browser}</span>
            </div>

            <div className="flex items-center gap-2">
              <span className="font-medium">Action:</span>
              <span className="capitalize">{task.action === TaskAction.Open ? '📂 Open' : '❌ Close'}</span>
            </div>

            {task.url && (
              <div className="flex items-center gap-2">
                <span className="font-medium">URL:</span>
                <span className="truncate max-w-md">{task.url}</span>
              </div>
            )}

            <div className="flex items-center gap-2">
              <span className="font-medium">Scheduled:</span>
              <span>{formatDate(task.scheduled_time)}</span>
            </div>

            {task.next_execution && (
              <div className="flex items-center gap-2">
                <span className="font-medium">Next Execution:</span>
                <span>{formatDate(task.next_execution)}</span>
              </div>
            )}

            {task.repeat_config && (
              <div className="flex items-center gap-2">
                <span className="font-medium">Repeat:</span>
                <span className="capitalize">{task.repeat_config.interval}</span>
              </div>
            )}
          </div>
        </div>

        <div className="flex gap-2 ml-4">
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
