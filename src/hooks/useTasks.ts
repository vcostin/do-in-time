import { useState, useEffect, useCallback } from 'react';
import { Task } from '../types/task';
import { TauriTaskService } from '../services/tauri-api';

const REFRESH_INTERVAL_MS = 5000; // Refresh every 5 seconds

export function useTasks() {
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

  // Initial load
  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  // Poll for updates every 5 seconds to keep UI in sync with task execution
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const data = await TauriTaskService.getAllTasks();
        setTasks(data);
      } catch (err) {
        // Silently fail on refresh errors to avoid spamming errors
        console.error('Failed to refresh tasks:', err);
      }
    }, REFRESH_INTERVAL_MS);

    return () => clearInterval(interval);
  }, []);

  return {
    tasks,
    loading,
    error,
    createTask,
    updateTask,
    deleteTask,
    refreshTasks: loadTasks,
  };
}
