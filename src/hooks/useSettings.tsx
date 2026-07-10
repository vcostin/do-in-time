import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { AppSettings } from '../types/task';
import { TauriTaskService } from '../services/tauri-api';

const DEFAULT_SETTINGS: AppSettings = {
  minimize_to_tray: false,
  start_minimized: false,
  show_notifications: false,
  auto_start: false,
  use_24_hour_clock: true,
};

interface SettingsContextValue {
  settings: AppSettings;
  loading: boolean;
  error: string | null;
  updateSettings: (newSettings: AppSettings) => Promise<void>;
  toggleSetting: (key: keyof AppSettings) => Promise<void>;
  refreshSettings: () => Promise<void>;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
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
    setError(null);
    const previousSettings = settings;

    try {
      setSettings(newSettings);
      const updated = await TauriTaskService.updateSettings(newSettings);

      if (updated.auto_start !== previousSettings.auto_start) {
        try {
          await TauriTaskService.applyAutoStart(updated.auto_start);
        } catch (autoStartErr) {
          console.warn('Auto-start setting could not be applied:', autoStartErr);
          const message = autoStartErr instanceof Error ? autoStartErr.message : 'Unknown error';
          setError(`Settings saved, but auto-start could not be configured: ${message}`);
        }
      }
    } catch (err) {
      setSettings(previousSettings);
      const message = err instanceof Error ? err.message : 'Failed to update settings';
      setError(message);
      throw new Error(message);
    }
  }, [settings]);

  const toggleSetting = useCallback(async (key: keyof AppSettings) => {
    await updateSettings({
      ...settings,
      [key]: !settings[key],
    });
  }, [settings, updateSettings]);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const value = useMemo(
    () => ({
      settings,
      loading,
      error,
      updateSettings,
      toggleSetting,
      refreshSettings: loadSettings,
    }),
    [settings, loading, error, updateSettings, toggleSetting, loadSettings],
  );

  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettings(): SettingsContextValue {
  const ctx = useContext(SettingsContext);
  if (!ctx) {
    throw new Error('useSettings must be used within SettingsProvider');
  }
  return ctx;
}
