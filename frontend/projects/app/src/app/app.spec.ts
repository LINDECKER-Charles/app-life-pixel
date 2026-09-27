import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { ApplicationInitStatus } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import axe from 'axe-core';
import en from '../../../../../i18n/en.json';
import languages from '../../../../../i18n/languages.json';
import { App } from './app';
import { appConfig } from './app.config';
import { routes } from './app.routes';
import { hostedOnly } from './platform/platform-guards';

const SERIOUS_IMPACTS = ['serious', 'critical'];

async function startApp(): Promise<ComponentFixture<App>> {
  TestBed.configureTestingModule({
    providers: [...appConfig.providers, provideHttpClientTesting()],
  });
  const initialization = TestBed.inject(ApplicationInitStatus).donePromise;
  const http = TestBed.inject(HttpTestingController);
  (await vi.waitFor(() => http.expectOne('/i18n/languages.json'))).flush(languages);
  (await vi.waitFor(() => http.expectOne('/i18n/en.json'))).flush(en);
  (await vi.waitFor(() => http.expectOne('/api/v1/auth/session'))).flush(
    { code: 'auth.unauthenticated', params: {} },
    { status: 401, statusText: 'Unauthorized' },
  );
  await initialization;
  const fixture = TestBed.createComponent(App);
  await fixture.whenStable();
  return fixture;
}

async function navigate(fixture: ComponentFixture<App>, url: string): Promise<void> {
  await TestBed.inject(Router).navigateByUrl(url);
  await fixture.whenStable();
}

/** The page entered last: Ionic's outlet keeps the former ones in its stack, before it. */
function page(fixture: ComponentFixture<App>): string | undefined {
  return fixture.nativeElement.querySelector('ion-router-outlet')?.lastElementChild?.localName;
}

/**
 * The one main landmark of the shell and the page shown — the pages the outlet keeps in its
 * stack are hidden —, once Ionic has rendered it.
 */
async function mainLandmark(root: HTMLElement): Promise<Element> {
  return vi.waitFor(() => {
    const shown = root.querySelector('ion-router-outlet')?.lastElementChild;
    const landmarks = [...root.querySelectorAll('main, [role="main"]')].filter(
      (landmark) => !landmark.closest('ion-router-outlet > *') || shown?.contains(landmark),
    );
    expect(landmarks).toHaveLength(1);
    return landmarks[0];
  });
}

describe('routes', () => {
  it('loads every page lazily', () => {
    const pages = routes.filter((route) => route.redirectTo === undefined);

    expect(pages.length).toBeGreaterThan(0);
    expect(pages.every((route) => route.loadComponent && !route.component)).toBe(true);
  });

  it('keeps the account, sign-in and support routes to the hosted app (desktop.md, T2)', () => {
    const hostedOnlyPaths = [
      'sign-in',
      'sign-up',
      'verify-email',
      'reset-password',
      'reset-password/confirm',
      'account',
      'support',
      'support/:requestId',
    ];

    for (const path of hostedOnlyPaths) {
      expect(routes.find((route) => route.path === path)?.canActivate).toContain(hostedOnly);
    }
    for (const path of ['editor', 'settings', 'library']) {
      expect(routes.find((route) => route.path === path)?.canActivate ?? []).not.toContain(
        hostedOnly,
      );
    }
  });
});

describe('App', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('fetches the catalogue of its language at start-up and renders from it', async () => {
    const fixture = await startApp();

    expect(fixture.nativeElement.querySelector('.brand')?.textContent).toBe(en['app.name']);
  });

  it('opens on a skip link that takes the focus into the page’s one main landmark', async () => {
    const fixture = await startApp();
    const root = fixture.nativeElement as HTMLElement;
    document.body.append(root);

    for (const path of ['/settings', '/editor']) {
      await navigate(fixture, path);
      const skipLink = root.querySelector<HTMLAnchorElement>('a[href]');
      const main = await mainLandmark(root);

      expect(skipLink?.textContent?.trim()).toBe(en['shell.skip_to_content']);
      skipLink?.focus();
      expect(document.activeElement).toBe(skipLink);

      skipLink?.click();

      expect(main.contains(document.activeElement)).toBe(true);
      expect(TestBed.inject(Router).url).toBe(path);
      // A shadow host with a negative tabindex would drop the whole page from the Tab order.
      expect(
        root.querySelectorAll('[tabindex="-1"]:is(ion-content, ion-router-outlet)'),
      ).toHaveLength(0);
    }
    root.remove();
  });

  it('gives every page exactly one main landmark', async () => {
    const fixture = await startApp();
    const root = fixture.nativeElement as HTMLElement;

    for (const path of ['/editor', '/settings', '/sign-in', '/no/such/page']) {
      await navigate(fixture, path);

      const main = await mainLandmark(root);
      const shown = root.querySelector('ion-router-outlet')?.lastElementChild;

      expect(shown?.contains(main)).toBe(true);
    }
  });

  it('resolves each path to its page', async () => {
    const fixture = await startApp();

    await navigate(fixture, '/');
    expect(TestBed.inject(Router).url).toBe('/editor');
    expect(page(fixture)).toBe('lp-editor-page');

    await navigate(fixture, '/settings');
    expect(page(fixture)).toBe('lp-settings-page');

    await navigate(fixture, '/no/such/page');
    expect(page(fixture)).toBe('lp-not-found-page');
  });

  it('titles the browser tab after the page', async () => {
    const fixture = await startApp();

    await navigate(fixture, '/settings');

    expect(document.title).toBe(`${en['settings.title']} – ${en['app.name']}`);
  });

  it('has no serious accessibility violation in the shell and the not-found page', async () => {
    const fixture = await startApp();
    await navigate(fixture, '/no/such/page');
    expect(fixture.nativeElement.querySelector('lp-not-found-page h1')).not.toBeNull();

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
