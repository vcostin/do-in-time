import { Navigate, Route, Routes } from 'react-router-dom';
import { AppLayout } from './components/AppLayout';
import { TaskListPage } from './pages/TaskListPage';
import { CalendarPage } from './pages/CalendarPage';
import { TaskFormPage } from './pages/TaskFormPage';
import { ActivityPage } from './pages/ActivityPage';
import { SettingsPage } from './pages/SettingsPage';

function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<TaskListPage />} />
        <Route path="calendar" element={<CalendarPage />} />
        <Route path="tasks/new" element={<TaskFormPage />} />
        <Route path="tasks/:taskId/edit" element={<TaskFormPage />} />
        <Route path="activity" element={<ActivityPage />} />
        <Route path="settings" element={<SettingsPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}

export default App;
