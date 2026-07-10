import { createHashRouter, Navigate, RouterProvider } from 'react-router-dom';
import { AppLayout } from './components/AppLayout';
import { TaskListPage } from './pages/TaskListPage';
import { CalendarPage } from './pages/CalendarPage';
import { TaskFormPage } from './pages/TaskFormPage';
import { ActivityPage } from './pages/ActivityPage';
import { SettingsPage } from './pages/SettingsPage';

const router = createHashRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <TaskListPage /> },
      { path: 'calendar', element: <CalendarPage /> },
      { path: 'tasks/new', element: <TaskFormPage /> },
      { path: 'tasks/:taskId/edit', element: <TaskFormPage /> },
      { path: 'activity', element: <ActivityPage /> },
      { path: 'settings', element: <SettingsPage /> },
      { path: '*', element: <Navigate to="/" replace /> },
    ],
  },
]);

function App() {
  return <RouterProvider router={router} />;
}

export default App;
