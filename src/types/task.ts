export enum BrowserType {
  Chrome = 'chrome',
  Firefox = 'firefox',
  Edge = 'edge',
  Safari = 'safari',
  Brave = 'brave',
  Opera = 'opera',
}

export enum TaskStatus {
  Active = 'active',
  Completed = 'completed',
  Failed = 'failed',
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
}

export interface SchedulerStatus {
  running: boolean;
}
