import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useTasks } from './hooks/useTasks';
import { useScheduler } from './hooks/useScheduler';
import { TaskForm } from './components/TaskForm';
import { TaskList } from './components/TaskList';
import { ScheduleCalendar } from './components/ScheduleCalendar';
import { SchedulerStatus } from './components/SchedulerStatus';
import { SettingsModal } from './components/SettingsModal';
import { ActivityModal } from './components/ActivityModal';
import { Task } from './types/task';

type MainView = 'list' | 'calendar';

function App() {
  const { tasks, loading, error, createTask, updateTask, deleteTask, setTaskPaused, getTaskExecutionLog } = useTasks();
  const { running, toggleScheduler } = useScheduler();
  const [showForm, setShowForm] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showActivity, setShowActivity] = useState(false);
  const [mainView, setMainView] = useState<MainView>('list');

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

  const handleCancel = () => {
    setShowForm(false);
    setEditingTask(null);
  };

  // Listen for 'open-settings' event from system tray
  useEffect(() => {
    const unlisten = listen('open-settings', () => {
      setShowSettings(true);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const viewToggle = (
    <div className="inline-flex items-center gap-0.5 p-0.5 rounded-lg bg-gray-200 dark:bg-gray-800 border border-gray-300 dark:border-gray-600">
      <button
        type="button"
        onClick={() => setMainView('list')}
        className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
          mainView === 'list'
            ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
            : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
        }`}
      >
        List
      </button>
      <button
        type="button"
        onClick={() => setMainView('calendar')}
        className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
          mainView === 'calendar'
            ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
            : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
        }`}
      >
        Calendar
      </button>
    </div>
  );

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
            <div className="flex items-center gap-3">
              <button
                onClick={() => setShowActivity(true)}
                className="px-3 py-2 text-sm text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                title="Activity across all tasks"
              >
                Activity
              </button>
              <button
                onClick={() => setShowSettings(true)}
                className="p-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
                title="Settings"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
              </button>
              <SchedulerStatus running={running} onToggle={toggleScheduler} />
            </div>
          </div>
        </header>

        <main>
          <div className="mb-6 flex flex-wrap items-center gap-3">
            {!showForm && (
              <button
                onClick={() => setShowForm(true)}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                + New Task
              </button>
            )}
            {!loading && !error && viewToggle}
          </div>

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
          ) : mainView === 'calendar' ? (
            <ScheduleCalendar tasks={tasks} onEditTask={handleEdit} />
          ) : (
            <TaskList
              tasks={tasks}
              onEdit={handleEdit}
              onDelete={handleDelete}
              onPause={handlePause}
              onResume={handleResume}
              onLoadLog={getTaskExecutionLog}
            />
          )}
        </main>
      </div>

      <SettingsModal
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
      />
      <ActivityModal
        isOpen={showActivity}
        onClose={() => setShowActivity(false)}
      />
    </div>
  );
}

export default App;
