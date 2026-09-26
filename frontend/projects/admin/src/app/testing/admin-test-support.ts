import {
  ApplicationInitStatus,
  EnvironmentProviders,
  inject,
  Injectable,
  provideAppInitializer,
  Provider,
  Type,
} from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideRouter, Router } from '@angular/router';
import {
  provideTransloco,
  Translation,
  TranslocoLoader,
  TranslocoService,
} from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import axe from 'axe-core';
import { firstValueFrom, Observable, of } from 'rxjs';
import { vi } from 'vitest';
import en from '../../../../../../i18n/en.json';
import type { ConsoleSession } from '../core/admin-types';
import { FileSaver } from '../core/file-saver';
import { SessionState } from '../core/session-state';
import { ChartHandle, ChartRenderer } from '../monitoring/chart-renderer';

/** A console of staging, whose monitoring shows staging and production. */
export const SESSION: ConsoleSession = {
  admin: { id: 'adm-1', email: 'ops@example.com' },
  csrfToken: 'csrf-1',
  environment: 'staging',
  monitoredEnvironments: ['staging', 'production'],
};

/** The English catalogue, without a request: the console's own loader fetches `/i18n/`. */
@Injectable()
class EnglishCatalogue implements TranslocoLoader {
  getTranslation(): Observable<Translation> {
    return of(en);
  }
}

export function provideTestI18n(): (Provider | EnvironmentProviders)[] {
  return [
    provideTransloco({
      config: { availableLangs: ['en'], defaultLang: 'en', prodMode: true },
      loader: EnglishCatalogue,
    }),
    provideTranslocoMessageformat(),
    provideAppInitializer(async () => {
      await firstValueFrom(inject(TranslocoService).load('en'));
    }),
  ];
}

/** A chart renderer that draws nothing: jsdom has no canvas. */
export class FakeChartRenderer extends ChartRenderer {
  readonly render = vi.fn((): Promise<ChartHandle> =>
    Promise.resolve({ resize: vi.fn(), destroy: vi.fn() }),
  );
}

/** A file saver that records what the page would download. */
export function fakeSaver(): { save: ReturnType<typeof vi.fn>; saveCsv: ReturnType<typeof vi.fn> } {
  return { save: vi.fn(), saveCsv: vi.fn() };
}

export interface PageOptions {
  readonly providers?: readonly (Provider | EnvironmentProviders)[];
  readonly inputs?: Readonly<Record<string, unknown>>;
  /** The address the page opens at, with its query. */
  readonly url?: string;
  readonly session?: ConsoleSession | null;
}

/**
 * Opens `component` signed in to `SESSION`, at `url`, with the English catalogue, a file saver
 * that saves nothing and a chart renderer that draws nothing, plus the page's own mocks.
 */
export async function openPage<T>(
  component: Type<T>,
  options: PageOptions = {},
): Promise<ComponentFixture<T>> {
  TestBed.configureTestingModule({
    providers: [
      provideRouter([{ path: '**', children: [] }]),
      provideTestI18n(),
      { provide: FileSaver, useValue: fakeSaver() },
      // Not `useClass`: a subclass would inherit ChartRenderer's own `providedIn` factory.
      { provide: ChartRenderer, useFactory: () => new FakeChartRenderer() },
      ...(options.providers ?? []),
    ],
  });
  await TestBed.inject(ApplicationInitStatus).donePromise;
  TestBed.inject(SessionState).set(options.session === undefined ? SESSION : options.session);
  await TestBed.inject(Router).navigateByUrl(options.url ?? '/');
  const fixture = TestBed.createComponent(component);
  for (const [name, value] of Object.entries(options.inputs ?? {})) {
    fixture.componentRef.setInput(name, value);
  }
  await fixture.whenStable();
  return fixture;
}

/** Waits for `assertion` to hold, running effects and change detection on each try. */
export async function settled(assertion: () => void): Promise<void> {
  await vi.waitFor(() => {
    TestBed.tick();
    assertion();
  });
}

/** The page's text, its white space collapsed. */
export function textOf(element: Element | null | undefined): string {
  return (element?.textContent ?? '').replace(/\s+/g, ' ').trim();
}

const SERIOUS_IMPACTS = ['serious', 'critical'];

/** The serious and critical accessibility violations axe finds in `element`. */
export async function seriousViolations(element: Element): Promise<axe.Result[]> {
  const { violations } = await axe.run(element);
  return violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? ''));
}
