export enum BrowserType {
  Chrome = 'chrome',
  Firefox = 'firefox',
  Edge = 'edge',
  Safari = 'safari',
  Brave = 'brave',
  Opera = 'opera',
  Chromium = 'chromium',
  LibreWolf = 'librewolf',
}

export const BROWSER_LABELS: Record<BrowserType, string> = {
  [BrowserType.Chrome]: 'Chrome',
  [BrowserType.Firefox]: 'Firefox',
  [BrowserType.Edge]: 'Edge',
  [BrowserType.Safari]: 'Safari',
  [BrowserType.Brave]: 'Brave',
  [BrowserType.Opera]: 'Opera',
  [BrowserType.Chromium]: 'Chromium',
  [BrowserType.LibreWolf]: 'LibreWolf',
};

export enum TaskStatus {
  Active = 'active',
  Completed = 'completed',
  Failed = 'failed',
  Disabled = 'disabled',
}

export enum RepeatInterval {
  Daily = 'daily',
  Weekly = 'weekly',
  Monthly = 'monthly',
}

export interface RepeatConfig {
  interval: RepeatInterval;
  end_after?: number | null;
  end_date?: string | null;
}

export interface Task {
  id?: number | null;
  name: string;
  browser: BrowserType;
  browser_profile?: string | null;
  url?: string | null;
  allow_close_all: boolean;
  start_time: string;
  close_time?: string | null;
  timezone: string;
  repeat_config?: RepeatConfig | null;
  execution_count: number;
  status: TaskStatus;
  next_open_execution?: string | null;
  next_close_execution?: string | null;
  last_error?: string | null;
  last_execution_at?: string | null;
}

export interface TaskExecutionLogEntry {
  id: number;
  task_id: number;
  action: string;
  success: boolean;
  message?: string | null;
  created_at: string;
}

export interface RecentExecutionLogEntry {
  id: number;
  task_id: number;
  task_name: string;
  action: string;
  success: boolean;
  message?: string | null;
  created_at: string;
}

export interface SchedulerStatus {
  running: boolean;
}

export interface AppSettings {
  minimize_to_tray: boolean;
  start_minimized: boolean;
  show_notifications: boolean;
  auto_start: boolean;
}
