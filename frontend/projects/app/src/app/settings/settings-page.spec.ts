import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { ApplicationInitStatus, DOCUMENT } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Config } from '@ionic/angular';
import axe from 'axe-core';
import en from '../../../../../../i18n/en.json';
import fr from '../../../../../../i18n/fr.json';
import languages from '../../../../../../i18n/languages.json';
import { appConfig } from '../app.config';
import { SettingsPage } from './settings-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

async function answer(url: string, body: object): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  (await vi.waitFor(() => http.expectOne(url))).flush(body);
}

async function openSettings(): Promise<ComponentFixture<SettingsPage>> {
  TestBed.configureTestingModule({
    providers: [...appConfig.providers, provideHttpClientTesting()],
  });
  const initialization = TestBed.inject(ApplicationInitStatus).donePromise;
  await answer('/i18n/languages.json', languages);
  await answer('/i18n/en.json', en);
  const http = TestBed.inject(HttpTestingController);
  (await vi.waitFor(() => http.expectOne('/api/v1/auth/session'))).flush(
    { code: 'auth.unauthenticated', params: {} },
    { status: 401, statusText: 'Unauthorized' },
  );
  await initialization;
  const fixture = TestBed.createComponent(SettingsPage);
  await fixture.whenStable();
  return fixture;
}

async function choose(fixture: ComponentFixture<SettingsPage>, selector: string): Promise<void> {
  const field: HTMLInputElement = fixture.nativeElement.querySelector(selector);
  field.checked = true;
  field.dispatchEvent(new Event('change'));
  await fixture.whenStable();
}

describe('SettingsPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('lists the languages and switches to one without reloading', async () => {
    const fixture = await openSettings();
    const select: HTMLSelectElement = fixture.nativeElement.querySelector('select');
    expect([...select.options].map((option) => option.textContent?.trim())).toEqual(
      languages.map((language) => language.name),
    );

    select.value = 'fr';
    select.dispatchEvent(new Event('change'));
    await answer('/i18n/fr.json', fr);
    await fixture.whenStable();

    expect(fixture.nativeElement.querySelector('h1')?.textContent).toBe(fr['settings.title']);
    expect(TestBed.inject(DOCUMENT).documentElement.lang).toBe('fr');
  });

  it('applies the theme chosen to the whole page', async () => {
    const fixture = await openSettings();
    const root = TestBed.inject(DOCUMENT).documentElement;
    expect(root.dataset['theme']).toBe('system');

    await choose(fixture, 'input[name="theme"][value="dark"]');
    expect(root.dataset['theme']).toBe('dark');

    await choose(fixture, 'input[name="theme"][value="light"]');
    expect(root.dataset['theme']).toBe('light');
  });

  it('reduces motion, Ionic animations included', async () => {
    const fixture = await openSettings();
    const root = TestBed.inject(DOCUMENT).documentElement;
    const ionic = TestBed.inject(Config);
    expect(root.dataset['motion']).toBe('system');
    expect(ionic.getBoolean('animated', true)).toBe(true);

    await choose(fixture, 'input[name="motion"][value="reduce"]');

    expect(root.dataset['motion']).toBe('reduce');
    expect(ionic.getBoolean('animated', true)).toBe(false);

    await choose(fixture, 'input[name="motion"][value="system"]');

    expect(ionic.getBoolean('animated', true)).toBe(true);
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openSettings();

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
