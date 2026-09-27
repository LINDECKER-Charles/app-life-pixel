import { TestBed } from '@angular/core/testing';
import {
  SupportApi,
  type SupportRequestMessage,
  type SupportRequestPage,
  type SupportRequestThread,
} from 'shared';
import { vi } from 'vitest';

/** A request of the account, as the server answers it. */
export const THREAD: SupportRequestThread = {
  id: 'r1',
  category: 'bug',
  status: 'new',
  hasScreenshot: false,
  createdAt: '2026-09-26T10:00:00Z',
  updatedAt: '2026-09-26T10:00:00Z',
  messages: [
    {
      id: 'm1',
      author: 'user',
      body: 'The canvas stays blank.',
      createdAt: '2026-09-26T10:00:00Z',
    },
  ],
};

/** The team's answer on `THREAD`. */
export const TEAM_MESSAGE: SupportRequestMessage = {
  id: 'm2',
  author: 'team',
  body: 'Could you try again?',
  createdAt: '2026-09-26T11:00:00Z',
};

/** The control the `<label>` reading `name` points at, resolved as assistive technology does. */
export function labelledControl<T extends HTMLElement>(root: HTMLElement, name: string): T {
  const label = [...root.querySelectorAll('label')].find(
    (candidate) => candidate.textContent?.trim() === name,
  );
  const control = label?.control;
  if (!control) {
    throw new Error(`No control is labelled ${name}`);
  }
  return control as T;
}

/** The text of each element describing `control`, in its `aria-describedby` order. */
export function descriptionOf(control: Element): string[] {
  const ids = control.getAttribute('aria-describedby')?.split(/\s+/) ?? [];
  return ids.map((id) => {
    const description = control.ownerDocument.getElementById(id);
    if (!description) {
      throw new Error(`No element has the id ${id}`);
    }
    return description.textContent?.trim() ?? '';
  });
}

/** A `SupportApi` whose every method is a mock, answering an empty list by default. */
export type MockSupportApi = Record<'create' | 'list' | 'get' | 'reply', ReturnType<typeof vi.fn>>;

/**
 * Provides a mocked `SupportApi` to the next `openAccountPage`: call it first, before the testing
 * module is instantiated.
 */
export function mockSupportApi(): MockSupportApi {
  const empty: SupportRequestPage = { items: [], nextCursor: null };
  const api: MockSupportApi = {
    create: vi.fn().mockResolvedValue(THREAD),
    list: vi.fn().mockResolvedValue(empty),
    get: vi.fn().mockResolvedValue(THREAD),
    reply: vi.fn().mockResolvedValue(THREAD.messages[0]),
  };
  TestBed.configureTestingModule({
    providers: [{ provide: SupportApi, useValue: api }],
  });
  return api;
}
