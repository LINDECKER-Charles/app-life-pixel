import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { ApplicationInitStatus, ChangeDetectionStrategy, Component, DOCUMENT } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { AvailableLanguages } from './available-languages';
import { PREFERRED_LANGUAGE, PreferredLanguageSource } from './preferred-language';
import { provideI18n } from './provide-i18n';

const LANGUAGES = [
  { code: 'en', name: 'English' },
  { code: 'fr', name: 'Français' },
];
const CATALOGUES = {
  en: { 'common.close': 'Close', 'test.frames': '{count, plural, one {# frame} other {# frames}}' },
  fr: {
    'common.close': 'Fermer',
    'test.frames': '{count, plural, one {# image} other {# images}}',
  },
};

@Component({
  selector: 'lp-i18n-probe',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `{{ 'common.close' | transloco }} / {{ 'test.frames' | transloco: { count: 0 } }}`,
})
class I18nProbe {}

function configure(preferred: PreferredLanguageSource[] = []): void {
  TestBed.configureTestingModule({
    providers: [
      provideHttpClient(),
      provideHttpClientTesting(),
      provideI18n(),
      ...preferred.map((source) => ({
        provide: PREFERRED_LANGUAGE,
        useValue: source,
        multi: true,
      })),
    ],
  });
}

async function answer(url: string, body: object): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  const request = await vi.waitFor(() => http.expectOne(url));
  request.flush(body);
}

async function start(initialLanguage: 'en' | 'fr'): Promise<void> {
  const initialization = TestBed.inject(ApplicationInitStatus).donePromise;
  await answer('/i18n/languages.json', LANGUAGES);
  await answer(`/i18n/${initialLanguage}.json`, CATALOGUES[initialLanguage]);
  await initialization;
}

describe('provideI18n', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('switches the language without reloading', async () => {
    configure();
    await start('en');
    const fixture = TestBed.createComponent(I18nProbe);
    await fixture.whenStable();
    expect(fixture.nativeElement.textContent).toBe('Close / 0 frames');

    TestBed.inject(TranslocoService).setActiveLang('fr');
    await answer('/i18n/fr.json', CATALOGUES.fr);
    await fixture.whenStable();

    expect(fixture.nativeElement.textContent).toBe('Fermer / 0 image');
    expect(TestBed.inject(DOCUMENT).documentElement.lang).toBe('fr');
  });

  it('starts in a known preference before the browser languages', async () => {
    configure([() => undefined, () => Promise.resolve('fr')]);
    await start('fr');

    expect(TestBed.inject(TranslocoService).getActiveLang()).toBe('fr');
    expect(TestBed.inject(AvailableLanguages).languages()).toEqual(LANGUAGES);
  });

  it('starts in English when the list of languages cannot be read', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
    configure([() => 'fr']);
    const initialization = TestBed.inject(ApplicationInitStatus).donePromise;
    const http = TestBed.inject(HttpTestingController);
    (await vi.waitFor(() => http.expectOne('/i18n/languages.json'))).flush('', {
      status: 503,
      statusText: 'Service Unavailable',
    });
    await answer('/i18n/en.json', CATALOGUES.en);
    await initialization;

    expect(TestBed.inject(AvailableLanguages).languages()).toEqual([]);
    expect(TestBed.inject(TranslocoService).getActiveLang()).toBe('en');
  });
});
