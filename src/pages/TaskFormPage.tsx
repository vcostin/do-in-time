import { useNavigate, useParams } from 'react-router-dom';
import { useTasks } from '../hooks/useTasks';
import { TaskForm } from '../components/TaskForm';
import { PageHeader } from '../components/PageHeader';
import { Task } from '../types/task';

export function TaskFormPage() {
  const navigate = useNavigate();
  const { taskId } = useParams();
  const { tasks, loading, createTask, updateTask } = useTasks();

  const editingId = taskId != null ? Number(taskId) : null;
  const isEdit = editingId != null && !Number.isNaN(editingId);
  const editingTask = isEdit ? tasks.find((t) => t.id === editingId) ?? null : null;

  const handleSubmit = async (task: Task) => {
    try {
      if (isEdit && editingId != null) {
        await updateTask(editingId, task);
      } else {
        await createTask(task);
      }
      navigate(-1);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to save task');
    }
  };

  const handleCancel = () => {
    navigate(-1);
  };

  if (loading && isEdit) {
    return (
      <div className="text-center py-12">
        <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        <p className="mt-4 text-gray-600 dark:text-gray-400">Loading task...</p>
      </div>
    );
  }

  if (isEdit && !loading && !editingTask) {
    return (
      <div>
        <PageHeader title="Task not found" backTo="/" backLabel="Back to list" />
        <p className="text-sm text-gray-600 dark:text-gray-400">
          No task with id {taskId}. It may have been deleted.
        </p>
      </div>
    );
  }

  return (
    <div>
      <PageHeader
        title={isEdit ? 'Edit task' : 'New task'}
        description={
          isEdit
            ? 'Update schedule, browser, or repeat settings'
            : 'Create a scheduled browser open/close'
        }
        backTo="/"
        backLabel="Back to list"
      />
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
        <TaskForm
          initialTask={editingTask}
          onSubmit={handleSubmit}
          onCancel={handleCancel}
        />
      </div>
    </div>
  );
}
