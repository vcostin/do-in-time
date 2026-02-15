import { useState, useEffect, useCallback } from 'react';
import { Task } from '../types/task';
import { TauriTaskService } from '../services/tauri-api';
import { listen } from '@tauri-apps/api/event';

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

  // Listen for task-updated events from backend
  useEffect(() => {
    const unlisten = listen<number>('task-updated', () => {
      // Refresh all tasks when any task is updated by the scheduler
      loadTasks();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [loadTasks]);

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
