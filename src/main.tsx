import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter } from "react-router-dom";
import App from "./App";
import { SettingsProvider } from "./hooks/useSettings";
import { TasksProvider } from "./hooks/useTasks";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <SettingsProvider>
        <TasksProvider>
          <App />
        </TasksProvider>
      </SettingsProvider>
    </HashRouter>
  </React.StrictMode>,
);
