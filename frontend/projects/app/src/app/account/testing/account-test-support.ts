import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { ApplicationInitStatus, signal, Type } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import type { MotionPreference, ThemePreference } from 'shared';
import { vi } from 'vitest';
import en from '../../../../../../../i18n/en.json';
import languages from '../../../../../../../i18n/languages.json';
import { appConfig } from '../../app.config';
import { PreferencesStore } from '../../settings/preferences-store';

/** A `PreferencesStore` that only ever answers 'en', for tests that need `SessionStore` alone. */
export class FakePreferences extends PreferencesStore {
  readonly language = signal('en').asReadonly();
  readonly theme = signal<ThemePreference>('system').asReadonly();
  readonly motion = signal<MotionPreference>('system').asReadonly();

  override setLanguage(): Promise<void> {
    return Promise.resolve();
  }

  override setTheme(): Promise<void> {
    return Promise.resolve();
  }

  override setMotion(): Promise<void> {
    return Promise.resolve();
  }
}

/**
 * Waits for `assertion` to hold, ticking effects each try: an `ApiClient` call settles a few
 * promises deep, more than one `whenStable()` reliably drains in this harness.
 */
export async function waitForEffects(assertion: () => void): Promise<void> {
  await vi.waitFor(() => {
    TestBed.tick();
    assertion();
  });
}

/** An account as the server answers it, for the pages that show or update one. */
export const ACCOUNT = {
  id: 'a1',
  email: 'lee@example.com',
  emailVerified: false,
  language: 'en',
  plan: 'free',
  storage: { usedBytes: 512, limitBytes: 1_000_000 },
  createdAt: '2024-01-01T00:00:00Z',
};

async function answer(url: string, body: object): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  (await vi.waitFor(() => http.expectOne(url))).flush(body);
}

async function answerSession(account: typeof ACCOUNT | null): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  const request = await vi.waitFor(() => http.expectOne('/api/v1/auth/session'));
  if (account) {
    request.flush({ account, csrfToken: 't0k' });
  } else {
    request.flush(
      { code: 'auth.unauthenticated', params: {} },
      { status: 401, statusText: 'Unauthorized' },
    );
  }
}

/**
 * Boots the app (`app.config.ts`'s providers) and creates `component`: every account page needs
 * the catalogue and the start-up session read answered first, whether signed in or not.
 */
export async function openAccountPage<T>(
  component: Type<T>,
  account: typeof ACCOUNT | null = null,
  inputs: Record<string, unknown> = {},
): Promise<ComponentFixture<T>> {
  TestBed.configureTestingModule({
    providers: [...appConfig.providers, provideHttpClientTesting()],
  });
  const initialization = TestBed.inject(ApplicationInitStatus).donePromise;
  await answer('/i18n/languages.json', languages);
  await answer('/i18n/en.json', en);
  await answerSession(account);
  await initialization;
  const fixture = TestBed.createComponent(component);
  for (const [name, value] of Object.entries(inputs)) {
    fixture.componentRef.setInput(name, value);
  }
  await fixture.whenStable();
  return fixture;
}
