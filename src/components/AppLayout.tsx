import { useEffect } from 'react';
import { NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { listen } from '@tauri-apps/api/event';
import { useScheduler } from '../hooks/useScheduler';
import { SchedulerStatus } from './SchedulerStatus';

const navClass = ({ isActive }: { isActive: boolean }) =>
  `px-3 py-1.5 text-sm rounded-md transition-colors ${
    isActive
      ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
      : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
  }`;

const headerLinkClass = ({ isActive }: { isActive: boolean }) =>
  `px-3 py-2 text-sm rounded-lg border transition-colors ${
    isActive
      ? 'bg-gray-100 dark:bg-gray-700 border-gray-400 dark:border-gray-500 text-gray-900 dark:text-white'
      : 'text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
  }`;

export function AppLayout() {
  const { running, toggleScheduler } = useScheduler();
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const showTaskToolbar = pathname === '/' || pathname === '/calendar';

  useEffect(() => {
    const unlisten = listen('open-settings', () => {
      navigate('/settings');
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [navigate]);

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900">
      <div className="sticky top-0 z-40 bg-gray-100/95 dark:bg-gray-900/95 backdrop-blur-sm border-b border-gray-200/80 dark:border-gray-700/80">
        <div className="container mx-auto px-4 pt-6 pb-4">
          <header className={showTaskToolbar ? 'mb-4' : ''}>
            <div className="flex items-center justify-between gap-4 flex-wrap">
              <div>
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                  Browser Scheduler
                </h1>
                <p className="text-gray-600 dark:text-gray-400 mt-2">
                  Schedule browsers to open and close at specific times
                </p>
              </div>
              <div className="flex items-center gap-3 flex-wrap">
                <NavLink to="/activity" className={headerLinkClass} title="Activity across all tasks">
                  Activity
                </NavLink>
                <NavLink
                  to="/settings"
                  className={({ isActive }) =>
                    `p-2 rounded-lg transition-colors ${
                      isActive
                        ? 'text-gray-900 dark:text-white bg-gray-200 dark:bg-gray-700'
                        : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                    }`
                  }
                  title="Settings"
                  aria-label="Settings"
                >
                  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                    />
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                    />
                  </svg>
                </NavLink>
                <SchedulerStatus running={running} onToggle={toggleScheduler} />
              </div>
            </div>
          </header>

          {showTaskToolbar && (
            <div className="flex flex-wrap items-center gap-3">
              <NavLink
                to="/tasks/new"
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
              >
                + New Task
              </NavLink>
              <div className="inline-flex items-center gap-0.5 p-0.5 rounded-lg bg-gray-200 dark:bg-gray-800 border border-gray-300 dark:border-gray-600">
                <NavLink to="/" end className={navClass}>
                  List
                </NavLink>
                <NavLink to="/calendar" className={navClass}>
                  Calendar
                </NavLink>
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="container mx-auto px-4 py-6">
        <main>
          <Outlet />
        </main>
      </div>
    </div>
  );
}
