import { createHashRouter, Navigate, RouterProvider } from 'react-router-dom';
import { AppLayout } from './components/AppLayout';
import { TaskListPage } from './pages/TaskListPage';

/**
 * Route-level code splitting via the data router's `lazy` API (not React.lazy).
 * Avoids a Suspense boundary under the sticky header, which can glitch stroked
 * SVGs in WebKitGTK when async chunks resolve.
 */
const router = createHashRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <TaskListPage /> },
      {
        path: 'calendar',
        lazy: async () => {
          const { CalendarPage } = await import('./pages/CalendarPage');
          return { Component: CalendarPage };
        },
      },
      {
        path: 'tasks/new',
        lazy: async () => {
          const { TaskFormPage } = await import('./pages/TaskFormPage');
          return { Component: TaskFormPage };
        },
      },
      {
        path: 'tasks/:taskId/edit',
        lazy: async () => {
          const { TaskFormPage } = await import('./pages/TaskFormPage');
          return { Component: TaskFormPage };
        },
      },
      {
        path: 'activity',
        lazy: async () => {
          const { ActivityPage } = await import('./pages/ActivityPage');
          return { Component: ActivityPage };
        },
      },
      {
        path: 'settings',
        lazy: async () => {
          const { SettingsPage } = await import('./pages/SettingsPage');
          return { Component: SettingsPage };
        },
      },
      { path: '*', element: <Navigate to="/" replace /> },
    ],
  },
]);

function App() {
  return <RouterProvider router={router} />;
}

export default App;
