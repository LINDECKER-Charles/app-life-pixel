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

describe('routes', () => {
  it('loads every page lazily', () => {
    const pages = routes.filter((route) => route.redirectTo === undefined);

    expect(pages.length).toBeGreaterThan(0);
    expect(pages.every((route) => route.loadComponent && !route.component)).toBe(true);
  });
});

describe('App', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('fetches the catalogue of its language at start-up and renders from it', async () => {
    const fixture = await startApp();

    expect(fixture.nativeElement.querySelector('.brand')?.textContent).toBe(en['app.name']);
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
