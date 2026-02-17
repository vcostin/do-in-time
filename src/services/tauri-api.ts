import { invoke } from '@tauri-apps/api/core';
import { Task, SchedulerStatus } from '../types/task';

export class TauriTaskService {
  static async getAllTasks(): Promise<Task[]> {
    return invoke<Task[]>('get_all_tasks');
  }

  static async getTask(id: number): Promise<Task> {
    return invoke<Task>('get_task', { id });
  }

  static async createTask(task: Task): Promise<Task> {
    return invoke<Task>('create_task', { task });
  }

  static async updateTask(id: number, task: Task): Promise<Task> {
    return invoke<Task>('update_task', { id, task });
  }

  static async deleteTask(id: number): Promise<void> {
    return invoke<void>('delete_task', { id });
  }

  static async startScheduler(): Promise<void> {
    return invoke<void>('start_scheduler');
  }

  static async stopScheduler(): Promise<void> {
    return invoke<void>('stop_scheduler');
  }

  static async getSchedulerStatus(): Promise<SchedulerStatus> {
    return invoke<SchedulerStatus>('get_scheduler_status');
  }
}
