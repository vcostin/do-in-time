export enum BrowserType {
  Chrome = 'chrome',
  Firefox = 'firefox',
  Edge = 'edge',
  Safari = 'safari',
  Brave = 'brave',
  Opera = 'opera',
}

export enum ExecutionAction {
  Open = 'open',
  Close = 'close',
}

export enum TaskStatus {
  Pending = 'pending',
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
  start_time: string;  // When to open browser
  close_time?: string | null;  // Optional: when to close browser
  timezone: string;
  repeat_config?: RepeatConfig | null;
  status: TaskStatus;
  created_at: string;
  updated_at: string;
  last_open_execution?: string | null;
  last_close_execution?: string | null;
  next_open_execution?: string | null;
  next_close_execution?: string | null;
}

export interface TaskExecution {
  id: number;
  task_id: number;
  executed_at: string;
  action: ExecutionAction;
  status: 'success' | 'failed';
  error_message?: string | null;
}

export interface SchedulerStatus {
  running: boolean;
}
