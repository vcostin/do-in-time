import { useNavigate } from 'react-router-dom';
import { useTasks } from '../hooks/useTasks';
import { TaskList } from '../components/TaskList';

export function TaskListPage() {
  const navigate = useNavigate();
  const { tasks, loading, error, deleteTask, setTaskPaused, getTaskExecutionLog } = useTasks();

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this task?')) {
      try {
        await deleteTask(id);
      } catch (err) {
        alert(err instanceof Error ? err.message : 'Failed to delete task');
      }
    }
  };

  const handlePause = async (id: number) => {
    try {
      await setTaskPaused(id, true);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to pause task');
    }
  };

  const handleResume = async (id: number) => {
    try {
      await setTaskPaused(id, false);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to resume task');
    }
  };

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
    <TaskList
      tasks={tasks}
      onEdit={(task) => task.id != null && navigate(`/tasks/${task.id}/edit`)}
      onDelete={handleDelete}
      onPause={handlePause}
      onResume={handleResume}
      onLoadLog={getTaskExecutionLog}
    />
  );
}
