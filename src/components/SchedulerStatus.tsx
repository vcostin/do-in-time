interface SchedulerStatusProps {
  running: boolean;
  onToggle: () => Promise<void>;
}

export function SchedulerStatus({ running, onToggle }: SchedulerStatusProps) {
  return (
    <div className="flex items-center gap-3">
      <div className="flex items-center gap-2">
        <div
          className={`w-3 h-3 rounded-full ${
            running ? 'bg-green-500 animate-pulse' : 'bg-gray-400'
          }`}
        />
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Scheduler {running ? 'Running' : 'Stopped'}
        </span>
      </div>
      <button
        onClick={onToggle}
        className={`px-4 py-2 rounded-lg font-medium transition-colors ${
          running
            ? 'bg-red-600 hover:bg-red-700 text-white'
            : 'bg-green-600 hover:bg-green-700 text-white'
        }`}
      >
        {running ? 'Stop' : 'Start'}
      </button>
    </div>
  );
}
