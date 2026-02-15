import { useState } from 'react';
import { useTasks } from './hooks/useTasks';
import { useScheduler } from './hooks/useScheduler';
import { TaskForm } from './components/TaskForm';
import { TaskList } from './components/TaskList';
import { SchedulerStatus } from './components/SchedulerStatus';
import { Task } from './types/task';

function App() {
  const { tasks, loading, error, createTask, updateTask, deleteTask } = useTasks();
  const { running, toggleScheduler } = useScheduler();
  const [showForm, setShowForm] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);

  const handleCreateOrUpdate = async (task: Task) => {
    try {
      if (editingTask && editingTask.id) {
        await updateTask(editingTask.id, task);
      } else {
        await createTask(task);
      }
      setShowForm(false);
      setEditingTask(null);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to save task');
    }
  };

  const handleEdit = (task: Task) => {
    setEditingTask(task);
    setShowForm(true);
  };

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this task?')) {
      try {
        await deleteTask(id);
      } catch (err) {
        alert(err instanceof Error ? err.message : 'Failed to delete task');
      }
    }
  };

  const handleCancel = () => {
    setShowForm(false);
    setEditingTask(null);
  };

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900">
      <div className="container mx-auto px-4 py-8">
        <header className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                Browser Scheduler
              </h1>
              <p className="text-gray-600 dark:text-gray-400 mt-2">
                Schedule browsers to open and close at specific times
              </p>
            </div>
            <SchedulerStatus running={running} onToggle={toggleScheduler} />
          </div>
        </header>

        <main>
          {!showForm && (
            <button
              onClick={() => setShowForm(true)}
              className="mb-6 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              + New Task
            </button>
          )}

          {showForm && (
            <div className="mb-6 bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
              <TaskForm
                initialTask={editingTask}
                onSubmit={handleCreateOrUpdate}
                onCancel={handleCancel}
              />
            </div>
          )}

          {loading ? (
            <div className="text-center py-12">
              <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
              <p className="mt-4 text-gray-600 dark:text-gray-400">Loading tasks...</p>
            </div>
          ) : error ? (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <p className="text-red-800 dark:text-red-200">Error: {error}</p>
            </div>
          ) : (
            <TaskList
              tasks={tasks}
              onEdit={handleEdit}
              onDelete={handleDelete}
            />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
