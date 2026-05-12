import { invoke } from '@tauri-apps/api/core';
import type { ContentNode, Question, QuestionOption } from './types';

// ── Helpers ───────────────────────────────────────────────────────────────────

function textBody(text: string): ContentNode[] {
  return [{ type: 'text', text, style: { bold: false, italic: false, underline: false, strikethrough: false } }];
}

// ── API ───────────────────────────────────────────────────────────────────────

export const api = {
  questions: {
    list: () =>
      invoke<Question[]>('list_questions'),

    addMultipleChoice: (p: {
      points:   number;
      outcomes: string[];
      body:     ContentNode[];
      options:  QuestionOption[];
      shuffle:  boolean;
    }) => invoke<string>('add_multiple_choice_question', { payload: p }),

    addTrueFalse: (p: {
      points:         number;
      outcomes:       string[];
      body:           ContentNode[];
      correct_answer: boolean;
    }) => invoke<string>('add_true_false_question', { payload: p }),

    addFillInBlank: (p: {
      outcomes: string[];
      body:     ContentNode[];
      blanks:   Array<{ id: string; accepted_answers: string[]; points: number }>;
    }) => invoke<string>('add_fill_in_blank_question', { payload: p }),

    addClassic: (p: {
      points:       number;
      outcomes:     string[];
      body:         ContentNode[];
      rubric:       Array<{ description: string; max_points: number }>;
      answer_space: { lines: number } | 'free';
    }) => invoke<string>('add_classic_question', { payload: p }),
  },

  // convenience: build a plain-text ContentNode body
  textBody,
};
