interface ConfirmLeaveModalProps {
  onStay: () => void;
  onLeave: () => void;
}

export function ConfirmLeaveModal({ onStay, onLeave }: ConfirmLeaveModalProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="absolute inset-0 bg-black/50" onClick={onStay} aria-hidden />
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="leave-dialog-title"
        aria-describedby="leave-dialog-desc"
        className="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6"
      >
        <h2
          id="leave-dialog-title"
          className="text-lg font-semibold text-gray-900 dark:text-white"
        >
          Discard unsaved changes?
        </h2>
        <p id="leave-dialog-desc" className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          You have unsaved changes on this task. If you leave now, those changes will be lost.
        </p>
        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={onStay}
            className="px-4 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
          >
            Stay
          </button>
          <button
            type="button"
            onClick={onLeave}
            className="px-4 py-2 text-sm rounded-lg bg-red-600 text-white hover:bg-red-700 transition-colors"
          >
            Leave without saving
          </button>
        </div>
      </div>
    </div>
  );
}
