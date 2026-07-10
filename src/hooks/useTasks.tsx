import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { Task, TaskExecutionLogEntry } from '../types/task';
import { TauriTaskService } from '../services/tauri-api';
import { listen } from '@tauri-apps/api/event';

interface TasksContextValue {
  tasks: Task[];
  loading: boolean;
  error: string | null;
  createTask: (task: Task) => Promise<Task>;
  updateTask: (id: number, task: Task) => Promise<Task>;
  deleteTask: (id: number) => Promise<void>;
  setTaskPaused: (id: number, paused: boolean) => Promise<Task>;
  getTaskExecutionLog: (id: number, limit?: number) => Promise<TaskExecutionLogEntry[]>;
  refreshTasks: () => Promise<void>;
}

const TasksContext = createContext<TasksContextValue | null>(null);

export function TasksProvider({ children }: { children: ReactNode }) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadTasks = useCallback(async () => {
    try {
      setLoading(true);
      const data = await TauriTaskService.getAllTasks();
      setTasks(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load tasks');
    } finally {
      setLoading(false);
    }
  }, []);

  const createTask = useCallback(async (task: Task): Promise<Task> => {
    try {
      const newTask = await TauriTaskService.createTask(task);
      setTasks((prev) => [...prev, newTask]);
      return newTask;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to create task';
      throw new Error(message);
    }
  }, []);

  const updateTask = useCallback(async (id: number, task: Task): Promise<Task> => {
    try {
      const updated = await TauriTaskService.updateTask(id, task);
      setTasks((prev) => prev.map((t) => (t.id === id ? updated : t)));
      return updated;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to update task';
      throw new Error(message);
    }
  }, []);

  const deleteTask = useCallback(async (id: number) => {
    try {
      await TauriTaskService.deleteTask(id);
      setTasks((prev) => prev.filter((t) => t.id !== id));
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to delete task';
      throw new Error(message);
    }
  }, []);

  const setTaskPaused = useCallback(async (id: number, paused: boolean): Promise<Task> => {
    try {
      const updated = await TauriTaskService.setTaskPaused(id, paused);
      setTasks((prev) => prev.map((t) => (t.id === id ? updated : t)));
      return updated;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to update task status';
      throw new Error(message);
    }
  }, []);

  const getTaskExecutionLog = useCallback(
    async (id: number, limit = 20): Promise<TaskExecutionLogEntry[]> => {
      return TauriTaskService.getTaskExecutionLog(id, limit);
    },
    [],
  );

  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  useEffect(() => {
    const unlisten = listen<number>('task-updated', () => {
      loadTasks();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadTasks]);

  const value = useMemo(
    () => ({
      tasks,
      loading,
      error,
      createTask,
      updateTask,
      deleteTask,
      setTaskPaused,
      getTaskExecutionLog,
      refreshTasks: loadTasks,
    }),
    [
      tasks,
      loading,
      error,
      createTask,
      updateTask,
      deleteTask,
      setTaskPaused,
      getTaskExecutionLog,
      loadTasks,
    ],
  );

  return <TasksContext.Provider value={value}>{children}</TasksContext.Provider>;
}

export function useTasks(): TasksContextValue {
  const ctx = useContext(TasksContext);
  if (!ctx) {
    throw new Error('useTasks must be used within TasksProvider');
  }
  return ctx;
}
