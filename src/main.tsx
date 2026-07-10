import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { SettingsProvider } from "./hooks/useSettings";
import { TasksProvider } from "./hooks/useTasks";
// Eager CSS: keep datepicker styles in the main bundle (not a lazy chunk inject).
import "react-datepicker/dist/react-datepicker.css";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SettingsProvider>
      <TasksProvider>
        <App />
      </TasksProvider>
    </SettingsProvider>
  </React.StrictMode>,
);
