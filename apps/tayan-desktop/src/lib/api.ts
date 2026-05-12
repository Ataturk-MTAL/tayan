import { invoke } from '@tauri-apps/api/core';
import type { Question } from './types';

export const api = {
  questions: {
    list: () => invoke<Question[]>('list_questions'),
  },
};
