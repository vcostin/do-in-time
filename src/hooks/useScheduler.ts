import { useState, useEffect, useCallback } from 'react';
import { TauriTaskService } from '../services/tauri-api';

export function useScheduler() {
  const [running, setRunning] = useState(false);
  const [loading, setLoading] = useState(true);

  const checkStatus = useCallback(async () => {
    try {
      const status = await TauriTaskService.getSchedulerStatus();
      setRunning(status.running);
    } catch (err) {
      console.error('Failed to get scheduler status:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  const startScheduler = useCallback(async () => {
    try {
      await TauriTaskService.startScheduler();
      setRunning(true);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to start scheduler';
      throw new Error(message);
    }
  }, []);

  const stopScheduler = useCallback(async () => {
    try {
      await TauriTaskService.stopScheduler();
      setRunning(false);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to stop scheduler';
      throw new Error(message);
    }
  }, []);

  const toggleScheduler = useCallback(async () => {
    if (running) {
      await stopScheduler();
    } else {
      await startScheduler();
    }
  }, [running, startScheduler, stopScheduler]);

  useEffect(() => {
    checkStatus();
    // Check status every 5 seconds
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, [checkStatus]);

  return {
    running,
    loading,
    startScheduler,
    stopScheduler,
    toggleScheduler,
  };
}
