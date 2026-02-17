import { useSettings } from '../hooks/useSettings';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface ToggleSwitchProps {
  enabled: boolean;
  onChange: () => void;
  label: string;
  description: string;
}

function ToggleSwitch({ enabled, onChange, label, description }: ToggleSwitchProps) {
  return (
    <div className="flex items-center justify-between py-4 border-b border-gray-200 dark:border-gray-700 last:border-0">
      <div className="flex-1 pr-4">
        <div className="text-sm font-medium text-gray-900 dark:text-white">
          {label}
        </div>
        <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
          {description}
        </div>
      </div>
      <button
        type="button"
        onClick={onChange}
        className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${
          enabled ? 'bg-blue-600' : 'bg-gray-200 dark:bg-gray-700'
        }`}
      >
        <span
          className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
            enabled ? 'translate-x-6' : 'translate-x-1'
          }`}
        />
      </button>
    </div>
  );
}

export function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const { settings, loading, error, toggleSetting } = useSettings();

  const handleToggle = async (key: keyof typeof settings) => {
    try {
      await toggleSetting(key);
    } catch (err) {
      // Error is already set in the hook, just prevent unhandled rejection
      console.error('Failed to toggle setting:', err);
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black bg-opacity-50"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            Settings
          </h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {loading ? (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            Loading settings...
          </div>
        ) : error ? (
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-4">
            <p className="text-sm text-red-800 dark:text-red-200">{error}</p>
          </div>
        ) : null}

        {!loading && (
          <div className="space-y-0">
            <ToggleSwitch
              enabled={settings.minimize_to_tray}
              onChange={() => handleToggle('minimize_to_tray')}
              label="Minimize to Tray"
              description="Hide to system tray instead of quitting when closing the window"
            />
            <ToggleSwitch
              enabled={settings.start_minimized}
              onChange={() => handleToggle('start_minimized')}
              label="Start Minimized"
              description="Launch the application hidden in the system tray"
            />
            <ToggleSwitch
              enabled={settings.show_notifications}
              onChange={() => handleToggle('show_notifications')}
              label="Show Notifications"
              description="Display desktop notifications when tasks are executed"
            />
            <ToggleSwitch
              enabled={settings.auto_start}
              onChange={() => handleToggle('auto_start')}
              label="Auto-Start"
              description="Launch the application automatically when the system starts"
            />
          </div>
        )}

        <div className="mt-6 flex justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
}
