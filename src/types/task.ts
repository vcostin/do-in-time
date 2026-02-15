export enum BrowserType {
  Chrome = 'chrome',
  Firefox = 'firefox',
  Edge = 'edge',
  Safari = 'safari',
  Brave = 'brave',
  Opera = 'opera',
}

export enum TaskAction {
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
  action: TaskAction;
  scheduled_time: string;
  timezone: string;
  repeat_config?: RepeatConfig | null;
  status: TaskStatus;
  created_at: string;
  updated_at: string;
  last_executed?: string | null;
  next_execution?: string | null;
}

export interface TaskExecution {
  id: number;
  task_id: number;
  executed_at: string;
  status: 'success' | 'failed';
  error_message?: string | null;
}

export interface SchedulerStatus {
  running: boolean;
}
