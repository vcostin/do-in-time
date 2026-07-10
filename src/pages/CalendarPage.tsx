import { useNavigate } from 'react-router-dom';
import { useTasks } from '../hooks/useTasks';
import { ScheduleCalendar } from '../components/ScheduleCalendar';

export function CalendarPage() {
  const navigate = useNavigate();
  const { tasks, loading, error } = useTasks();

  if (loading) {
    return (
      <div className="text-center py-12">
        <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        <p className="mt-4 text-gray-600 dark:text-gray-400">Loading tasks...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <p className="text-red-800 dark:text-red-200">Error: {error}</p>
      </div>
    );
  }

  return (
    <ScheduleCalendar
      tasks={tasks}
      onEditTask={(task) => task.id != null && navigate(`/tasks/${task.id}/edit`)}
    />
  );
}
