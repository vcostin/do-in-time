import { useState, useEffect, useCallback } from 'react';
import { AppSettings } from '../types/task';
import { TauriTaskService } from '../services/tauri-api';

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>({
    minimize_to_tray: false,
    start_minimized: false,
    show_notifications: false,
    auto_start: false,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadSettings = useCallback(async () => {
    try {
      setLoading(true);
      const data = await TauriTaskService.getSettings();
      setSettings(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setLoading(false);
    }
  }, []);

  const updateSettings = useCallback(async (newSettings: AppSettings) => {
    // Clear any previous errors
    setError(null);

    // Save previous settings for rollback
    const previousSettings = settings;

    try {
      // Optimistically update UI
      setSettings(newSettings);

      // Save to database
      const updated = await TauriTaskService.updateSettings(newSettings);

      // Apply auto-start setting only if it changed
      if (updated.auto_start !== previousSettings.auto_start) {
        try {
          await TauriTaskService.applyAutoStart(updated.auto_start);
        } catch (autoStartErr) {
          // Auto-start might fail due to permissions or platform support
          // Log but don't fail the entire operation since DB update succeeded
          console.warn('Auto-start setting could not be applied:', autoStartErr);

          // Show a non-blocking warning
          const message = autoStartErr instanceof Error ? autoStartErr.message : 'Unknown error';
          setError(`Settings saved, but auto-start could not be configured: ${message}`);
        }
      }
    } catch (err) {
      // Rollback on database error
      setSettings(previousSettings);

      const message = err instanceof Error ? err.message : 'Failed to update settings';
      setError(message);
      throw new Error(message);
    }
  }, [settings]);

  const toggleSetting = useCallback(async (key: keyof AppSettings) => {
    const newSettings = {
      ...settings,
      [key]: !settings[key],
    };
    await updateSettings(newSettings);
  }, [settings, updateSettings]);

  // Initial load
  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  return {
    settings,
    loading,
    error,
    updateSettings,
    toggleSetting,
    refreshSettings: loadSettings,
  };
}
